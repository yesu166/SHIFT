import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Activity, Archive, ArrowLeftRight, ArrowRight, ArrowUpRight, Check, CheckCircle2,
  ChevronRight, Clock3, Command, FileArchive, FileClock, FileText, FolderOpen, HardDrive,
  Home, Layers3, LockKeyhole, Moon, MoreHorizontal, Plus, Settings, ShieldCheck, Sparkles,
  Sun, Trash2, Upload, X, Zap, type LucideIcon,
} from "lucide-react";
import { formatBytes, formatSignedBytes } from "./lib/format";
import { getAvailableOperations, type OperationDefinition } from "./lib/operations";

type PageId = "home" | "convert" | "organize" | "compress" | "edit" | "protect" | "history" | "settings";
type ThemeMode = "light" | "dark";
interface QueuedFile { id: string; name: string; path?: string; size?: number }
interface ZipResult { outputPath: string; inputBytes: number; archiveBytes: number; bytesSaved: number; filesArchived: number }
interface AppInfo { outputFolder: string; appVersion: string }
interface JobRecord extends ZipResult { id: string; completedAt: string; status: "done" }
interface NavItem { id: PageId; label: string; icon: LucideIcon }

const NAV_ITEMS: NavItem[] = [
  { id: "home", label: "Home", icon: Home },
  { id: "convert", label: "Convert", icon: ArrowLeftRight },
  { id: "organize", label: "Organize", icon: Layers3 },
  { id: "compress", label: "Compress", icon: Archive },
  { id: "edit", label: "Edit PDF", icon: FileText },
  { id: "protect", label: "Protect", icon: ShieldCheck },
  { id: "history", label: "Batch & History", icon: FileClock },
  { id: "settings", label: "Settings", icon: Settings },
];

const PLANNED: Record<string, { eyebrow: string; title: string; description: string; engine: string }> = {
  convert: {
    eyebrow: "CONVERSION WORKSPACE", title: "Convert without guesswork.",
    description: "SHIFT only offers a conversion when a real engine can produce and validate the requested output.",
    engine: "Office-to-PDF engine integration is not connected yet.",
  },
  organize: {
    eyebrow: "PDF WORKSPACE", title: "Put pages in the right order.",
    description: "Merge, split, rotate, extract, reorder and remove PDF pages without touching the original file.",
    engine: "PDF structural operations are not enabled in this build.",
  },
  edit: {
    eyebrow: "PDF WORKSPACE", title: "Make the small changes count.",
    description: "Page numbers, watermarks and metadata edits need a real PDF-writing implementation.",
    engine: "PDF writing and preview are not connected yet.",
  },
  protect: {
    eyebrow: "PRIVACY WORKSPACE", title: "Keep documents yours.",
    description: "Password tools appear only when encryption and output validation are implemented safely.",
    engine: "PDF encryption integration is not connected yet.",
  },
};

function makeId(): string {
  return globalThis.crypto?.randomUUID?.() ?? String(Date.now()) + "-" + Math.random().toString(16).slice(2);
}
function leafName(path: string): string { return path.split(/[\\/]/).pop() || path; }
function uniqueFiles(existing: QueuedFile[], incoming: QueuedFile[]): QueuedFile[] {
  const seen = new Set(existing.map((file) => file.path ?? file.name));
  const next = [...existing];
  for (const item of incoming) {
    const key = item.path ?? item.name;
    if (!seen.has(key)) { seen.add(key); next.push(item); }
  }
  return next;
}

function App() {
  const [page, setPage] = useState<PageId>("home");
  const [files, setFiles] = useState<QueuedFile[]>([]);
  const [archiveName, setArchiveName] = useState("");
  const [isDragging, setIsDragging] = useState(false);
  const [busy, setBusy] = useState(false);
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [toast, setToast] = useState<{ message: string; kind: "success" | "error" | "info" } | null>(null);
  const [wheelOpen, setWheelOpen] = useState(false);
  const [theme, setTheme] = useState<ThemeMode>(() => localStorage.getItem("shift.theme") === "dark" ? "dark" : "light");
  const [reducedMotion, setReducedMotion] = useState(() => localStorage.getItem("shift.reducedMotion") === "true");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const isDesktop = isTauri();

  const availableOperations = useMemo(() => getAvailableOperations(files.map((file) => file.name)), [files]);
  const canCreateZip = isDesktop && files.length > 0 && files.every((file) => Boolean(file.path)) && !busy;

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("shift.theme", theme);
  }, [theme]);
  useEffect(() => {
    document.documentElement.dataset.reducedMotion = String(reducedMotion);
    localStorage.setItem("shift.reducedMotion", String(reducedMotion));
  }, [reducedMotion]);
  useEffect(() => {
    let active = true;
    if (isDesktop) {
      void invoke<AppInfo>("get_app_info").then((info) => { if (active) setAppInfo(info); })
        .catch((error: unknown) => { if (active) setToast({ message: "Unable to read SHIFT settings: " + String(error), kind: "error" }); });
    }
    return () => { active = false; };
  }, [isDesktop]);
  useEffect(() => {
    if (!isDesktop) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void import("@tauri-apps/api/webview").then(({ getCurrentWebview }) =>
      getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") setIsDragging(true);
        else if (event.payload.type === "leave") setIsDragging(false);
        else if (event.payload.type === "drop") {
          setIsDragging(false);
          setFiles((current) => uniqueFiles(current, event.payload.paths.map((path) => ({ id: makeId(), name: leafName(path), path }))));
          setPage("compress");
        }
      }),
    ).then((stop) => { if (cancelled) stop(); else unlisten = stop; })
      .catch((error: unknown) => setToast({ message: "File drop support could not start: " + String(error), kind: "error" }));
    return () => { cancelled = true; unlisten?.(); };
  }, [isDesktop]);
  useEffect(() => {
    if (!wheelOpen) return;
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") setWheelOpen(false); };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [wheelOpen]);
  useEffect(() => {
    if (!toast) return;
    const timeout = window.setTimeout(() => setToast(null), 5200);
    return () => window.clearTimeout(timeout);
  }, [toast]);

  const addBrowserFiles = useCallback((browserFiles: FileList | File[]) => {
    const incoming = Array.from(browserFiles).map((file) => ({ id: makeId(), name: file.name, size: file.size }));
    setFiles((current) => uniqueFiles(current, incoming));
    setPage("compress");
    if (!isDesktop && incoming.length) setToast({ message: "Preview mode only. Open the Windows desktop build to process native file paths.", kind: "info" });
  }, [isDesktop]);

  const chooseFiles = useCallback(async () => {
    if (!isDesktop) { fileInputRef.current?.click(); return; }
    try {
      const selected = await open({ multiple: true, directory: false, title: "Choose files for your SHIFT archive" });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      setFiles((current) => uniqueFiles(current, paths.map((path) => ({ id: makeId(), name: leafName(path), path }))));
      setPage("compress");
    } catch (error) {
      setToast({ message: "The file picker failed: " + String(error), kind: "error" });
    }
  }, [isDesktop]);

  const removeFile = useCallback((id: string) => setFiles((current) => current.filter((file) => file.id !== id)), []);
  const createZip = useCallback(async () => {
    if (!canCreateZip) {
      setToast({ message: isDesktop ? "Add at least one file with a native local path." : "Open SHIFT as a Windows desktop app to create archives.", kind: "info" });
      return;
    }
    setBusy(true);
    setToast(null);
    try {
      const result = await invoke<ZipResult>("create_zip", { inputs: files.map((file) => file.path), archiveName: archiveName.trim() || null });
      setJobs((current) => [{ ...result, id: makeId(), completedAt: new Date().toISOString(), status: "done" }, ...current].slice(0, 30));
      setFiles([]);
      setArchiveName("");
      setToast({ message: "Archive verified and saved.", kind: "success" });
      setPage("history");
    } catch (error) {
      setToast({ message: "Archive failed: " + String(error), kind: "error" });
    } finally { setBusy(false); }
  }, [archiveName, canCreateZip, files, isDesktop]);

  const openOutputFolder = useCallback(async () => {
    if (!isDesktop) { setToast({ message: "Output folders are available in the Windows desktop build.", kind: "info" }); return; }
    try { await invoke("open_output_folder"); }
    catch (error) { setToast({ message: "Could not open the output folder: " + String(error), kind: "error" }); }
  }, [isDesktop]);

  const selectOperation = useCallback((operation: OperationDefinition) => {
    setWheelOpen(false);
    if (operation.id === "create-zip") {
      setPage("compress");
      if (files.length === 0) void chooseFiles();
      else void createZip();
    } else {
      setToast({ message: operation.unavailableReason ?? "This operation is not enabled.", kind: "info" });
    }
  }, [chooseFiles, createZip, files.length]);

  const currentTitle = NAV_ITEMS.find((item) => item.id === page)?.label ?? "Home";

  return <div className="app-shell">
    <aside className="sidebar">
      <button className="brand-lockup" type="button" onClick={() => setPage("home")} aria-label="SHIFT home">
        <span className="brand-mark"><span /></span><span className="brand-name">shift<span>.</span><small>DOCUMENT TOOLBOX</small></span>
      </button>
      <div className="workspace-label">WORKSPACE <span>01</span></div>
      <nav className="primary-nav" aria-label="Main navigation">
        {NAV_ITEMS.slice(0, 6).map((item) => {
          const Icon = item.icon;
          return <button key={item.id} className={"nav-item" + (page === item.id ? " active" : "")} onClick={() => setPage(item.id)} type="button">
            <Icon size={17} strokeWidth={1.8} /><span>{item.label}</span>{item.id === "compress" && <i className="nav-dot" aria-label="Available" />}
          </button>;
        })}
      </nav>
      <div className="sidebar-spacer" />
      <div className="sidebar-note"><div className="note-icon"><ShieldCheck size={17} /></div><strong>Your files stay yours.</strong><p>Processing is local. Nothing is uploaded to a cloud service.</p><div className="privacy-status"><i /> LOCAL-FIRST BY DESIGN</div></div>
      <div className="sidebar-bottom">
        <button className={"nav-item" + (page === "history" ? " active" : "")} onClick={() => setPage("history")} type="button"><Clock3 size={17} /><span>Batch &amp; History</span>{jobs.length > 0 && <b className="count-pill">{jobs.length}</b>}</button>
        <button className={"nav-item" + (page === "settings" ? " active" : "")} onClick={() => setPage("settings")} type="button"><Settings size={17} /><span>Settings</span></button>
        <div className="profile-row"><div className="profile-avatar">S</div><div className="profile-copy"><strong>SHIFT Desktop</strong><span>Version {appInfo?.appVersion ?? "0.1.0"}</span></div><MoreHorizontal size={18} className="muted-icon" /></div>
      </div>
    </aside>

    <main className="main-area">
      <header className="topbar"><div className="breadcrumbs"><span>SHIFT</span><ChevronRight size={14} /><strong>{currentTitle}</strong></div><div className="topbar-actions"><div className={"local-pill" + (isDesktop ? " connected" : "")}><i />{isDesktop ? "LOCAL MODE" : "PREVIEW MODE"}</div><button className="icon-button" onClick={() => setWheelOpen((v) => !v)} type="button" aria-label="Open quick action wheel" aria-expanded={wheelOpen}><Command size={18} /></button></div></header>

      {page === "home" && <div className="page page-home">
        <div className="page-intro"><div><div className="eyebrow"><Sparkles size={14} /> A LITTLE LESS FILE CHAOS</div><h1>Your files.<br /><span>Sorted.</span></h1><p className="intro-copy">The everyday document toolkit that gets out of your way. Simple actions, clean outputs, no cloud detours.</p></div>
          <div className="intro-orbit" aria-hidden="true"><div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="orbit-center"><span className="brand-mark"><span /></span></div><span className="orbit-file orbit-a"><FileText size={19} /></span><span className="orbit-file orbit-b"><FileArchive size={19} /></span><span className="orbit-file orbit-c"><CheckCircle2 size={19} /></span></div>
        </div>
        <div className={"drop-card" + (isDragging ? " drag-active" : "")} onDragEnter={(e) => { e.preventDefault(); setIsDragging(true); }} onDragOver={(e) => e.preventDefault()} onDragLeave={(e) => { if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setIsDragging(false); }} onDrop={(e) => { e.preventDefault(); setIsDragging(false); addBrowserFiles(e.dataTransfer.files); }}>
          <input ref={fileInputRef} className="hidden-file-input" type="file" multiple onChange={(e) => { if (e.target.files) addBrowserFiles(e.target.files); e.currentTarget.value = ""; }} />
          <div className="drop-inner"><div className="upload-illustration"><div className="upload-sheet"><FileText size={23} /><span /></div><div className="upload-arrow"><Upload size={18} /></div></div>
            <div className="drop-copy"><h2>Drop files to get moving</h2><p>PDF, Word, Excel, PowerPoint, images, folders — bundle anything into a ZIP.</p><div className="drop-actions"><button className="button button-primary" type="button" onClick={chooseFiles}>Choose files <ArrowRight size={16} /></button><span>or drag them here</span></div></div>
            <div className="drop-format-row"><span>PDF</span><span>DOCX</span><span>XLSX</span><span>PPTX</span><span>+ ANY FILE</span></div>
          </div>
        </div>
        <div className="section-heading"><div><span className="eyebrow">YOUR WORKSPACE</span><h2>Quick actions</h2></div><button className="text-button" type="button" onClick={() => setPage("compress")}>View all actions <ArrowRight size={15} /></button></div>
        <div className="action-grid">
          <button className="action-card action-card-featured" type="button" onClick={() => setPage("compress")}><span className="action-symbol symbol-orange"><Archive size={20} /></span><span className="action-copy"><strong>Compress files to ZIP</strong><small>Bundle files and folders into one archive.</small><em><i /> READY TO USE</em></span><ArrowUpRight className="action-arrow" size={18} /></button>
          <div className="action-card action-card-muted" aria-disabled="true"><span className="action-symbol symbol-neutral"><Activity size={20} /></span><span className="action-copy"><strong>Optimize a PDF</strong><small>Structural optimization with measured savings.</small><em>ENGINE NOT CONNECTED</em></span><span className="coming-soon">SOON</span></div>
          <div className="action-card action-card-muted" aria-disabled="true"><span className="action-symbol symbol-neutral"><Layers3 size={20} /></span><span className="action-copy"><strong>Merge PDFs</strong><small>Bring pages together in the right order.</small><em>NOT ENABLED YET</em></span><span className="coming-soon">SOON</span></div>
        </div>
        <div className="bottom-grid">
          <div className="recent-panel"><div className="section-heading compact"><div><span className="eyebrow">ACTIVITY</span><h2>Recent work</h2></div><b className="panel-count">{jobs.length.toString().padStart(2, "0")}</b></div>
            {jobs.length === 0 ? <div className="empty-recent"><span className="empty-icon"><Clock3 size={18} /></span><div><strong>Nothing here. Yet.</strong><p>Your completed tasks will show up after the first run.</p></div></div> : <div className="recent-work-row"><span className="recent-work-icon"><Archive size={17} /></span><div><strong>{jobs.length} completed archive{jobs.length === 1 ? "" : "s"}</strong><p>See Batch &amp; History for output details.</p></div><CheckCircle2 className="success-icon" size={17} /></div>}
          </div>
          <div className="destination-panel"><div className="destination-top"><span className="destination-icon"><FolderOpen size={18} /></span><span className="eyebrow">DEFAULT DESTINATION</span><i /></div><h3>All in one place.</h3><p>{appInfo?.outputFolder ?? "Documents / SHIFT / Archives"}</p><button className="text-button" type="button" onClick={openOutputFolder}>Open folder <ArrowUpRight size={14} /></button></div>
        </div>
        {!isDesktop && <div className="preview-banner"><HardDrive size={16} /><span><strong>Web preview only.</strong> Native file access and ZIP creation require the Windows desktop build.</span></div>}
      </div>}

      {page === "compress" && <div className="page">
        <div className="page-title-row"><div><div className="eyebrow"><Archive size={14} /> ARCHIVE WORKSPACE</div><h1>Compress files.</h1><p>One neat archive. Originals left exactly where they are.</p></div><button className="button button-secondary" onClick={openOutputFolder} type="button"><FolderOpen size={16} /> Output folder</button></div>
        <div className="compress-layout">
          <section className="workspace-card"><div className="card-topline"><span className="step-number">01</span><div><h2>Add your files</h2><p>Mix formats freely. Folders and nested files are supported.</p></div><button className="icon-button subtle" type="button" onClick={chooseFiles} title="Add files" aria-label="Add files"><Plus size={19} /></button></div>
            <input ref={fileInputRef} className="hidden-file-input" type="file" multiple onChange={(e) => { if (e.target.files) addBrowserFiles(e.target.files); e.currentTarget.value = ""; }} />
            <div className="queue-summary"><span>{files.length} ITEM{files.length === 1 ? "" : "S"} SELECTED</span><span>{formatBytes(files.reduce((sum, file) => sum + (file.size ?? 0), 0))} known size</span></div>
            {files.length === 0 ? <button className="empty-queue" type="button" onClick={chooseFiles}><span className="empty-queue-icon"><Upload size={22} /></span><strong>Choose files or drop them into SHIFT</strong><small>Files are read from your machine only when you create the archive.</small><span className="empty-queue-cta">Browse local files <ChevronRight size={15} /></span></button> :
              <div className="file-list">{files.map((file, index) => <div className="file-row" key={file.id}><span className="file-type-icon">{(file.name.split(".").pop() || "FILE").slice(0, 4).toUpperCase()}</span><span className="file-meta"><strong>{file.name}</strong><small>{file.path ? "Local file" : "Preview only"}{file.size !== undefined ? " · " + formatBytes(file.size) : ""}</small></span><span className="file-index">{String(index + 1).padStart(2, "0")}</span><button className="remove-file" type="button" onClick={() => removeFile(file.id)} aria-label={"Remove " + file.name}><Trash2 size={15} /></button></div>)}<button className="add-more-button" type="button" onClick={chooseFiles}><Plus size={16} /> Add more files</button></div>}
            <div className="privacy-inline"><ShieldCheck size={16} /><span><strong>Private by default.</strong> SHIFT never uploads files for this operation.</span></div>
          </section>
          <aside className="options-column"><section className="workspace-card options-card"><div className="card-topline"><span className="step-number">02</span><div><h2>Archive options</h2><p>Keep it simple. Keep it safe.</p></div></div>
            <label className="field-label" htmlFor="archive-name">ARCHIVE NAME</label><div className="archive-name-field"><Archive size={17} /><input id="archive-name" value={archiveName} onChange={(e) => setArchiveName(e.target.value)} maxLength={80} placeholder="SHIFT Archive" /><span>.zip</span></div>
            <div className="option-meta"><span>Compression</span><strong>DEFLATE · Level 6</strong></div><div className="option-meta"><span>Output safety</span><strong><ShieldCheck size={14} /> No overwrite</strong></div><div className="option-meta"><span>Validation</span><strong><CheckCircle2 size={14} /> Read all entries</strong></div>
            <div className="output-path-block"><span className="field-label">SAVE LOCATION</span><div className="path-line"><FolderOpen size={16} /><span>{appInfo?.outputFolder ?? "Documents / SHIFT / Archives"}</span></div></div>
            <button className="button button-primary create-archive-button" type="button" onClick={createZip} disabled={!canCreateZip || busy || !availableOperations.some((op) => op.id === "create-zip")}>{busy ? <><span className="spinner" /> Creating and validating…</> : <><Archive size={17} /> Create ZIP archive <ArrowRight size={17} /></>}</button>
            {!isDesktop && <p className="inline-warning"><HardDrive size={14} /> Run the desktop app to access local paths.</p>}
            {isDesktop && files.length > 0 && !files.every((file) => Boolean(file.path)) && <p className="inline-warning"><HardDrive size={14} /> Re-select these items in the desktop picker to use native paths.</p>}
          </section><div className="safety-note"><LockKeyhole size={16} /><p><strong>Originals stay untouched.</strong><br />SHIFT writes a separate archive and validates it before reporting success.</p></div></aside>
        </div>
      </div>}

      {page === "history" && <div className="page"><div className="page-title-row"><div><div className="eyebrow"><FileClock size={14} /> ACTIVITY LOG</div><h1>Work, accounted for.</h1><p>Completed tasks from this session. No document contents are stored.</p></div><button className="button button-secondary" onClick={openOutputFolder} type="button"><FolderOpen size={16} /> Output folder</button></div>
        <section className="history-card"><div className="history-table-head"><span>OUTPUT</span><span>CONTENTS</span><span>SIZE CHANGE</span><span>STATUS</span></div>
          {jobs.length === 0 ? <div className="history-empty"><span className="empty-icon"><Clock3 size={20} /></span><h2>Your first completed job goes here.</h2><p>Run Compress files to ZIP and SHIFT will show its validated output here.</p></div> : jobs.map((job) => <div className="history-row" key={job.id}><span className="history-output"><span className="file-type-icon">ZIP</span><span><strong>{leafName(job.outputPath)}</strong><small>{new Date(job.completedAt).toLocaleString()}</small></span></span><span className="history-stat">{job.filesArchived} entries<br /><small>{formatBytes(job.inputBytes)} original</small></span><span className={"history-stat " + (job.bytesSaved < 0 ? "warning-text" : "positive-text")}>{formatSignedBytes(job.bytesSaved)}<br /><small>{formatBytes(job.archiveBytes)} output</small></span><span className="history-success"><CheckCircle2 size={16} /> Verified</span></div>)}
        </section>
      </div>}

      {page === "settings" && <div className="page"><div className="page-title-row"><div><div className="eyebrow"><Settings size={14} /> PREFERENCES</div><h1>Make SHIFT yours.</h1><p>Settings that actually change behavior. No decorative switches.</p></div></div>
        <div className="settings-layout">
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><Sun size={18} /></span><div><h2>Appearance</h2><p>Choose the workspace contrast that works for you.</p></div></div>
            <div className="theme-picker" role="group" aria-label="Color theme"><button className={"theme-choice" + (theme === "light" ? " selected" : "")} type="button" onClick={() => setTheme("light")}><span className="theme-swatch theme-light"><i /><i /><i /></span><span>Light</span>{theme === "light" && <Check size={16} />}</button><button className={"theme-choice" + (theme === "dark" ? " selected" : "")} type="button" onClick={() => setTheme("dark")}><span className="theme-swatch theme-dark"><i /><i /><i /></span><span>Dark</span>{theme === "dark" && <Check size={16} />}</button></div>
            <div className="settings-divider" /><div className="settings-row"><span className="settings-icon">{reducedMotion ? <Moon size={18} /> : <Zap size={18} />}</span><span className="settings-row-copy"><strong>Reduce motion</strong><small>Limit interface transitions and movement.</small></span><button className={"toggle" + (reducedMotion ? " on" : "")} type="button" role="switch" aria-checked={reducedMotion} onClick={() => setReducedMotion(!reducedMotion)} aria-label="Reduce motion"><span /></button></div>
          </section>
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><HardDrive size={18} /></span><div><h2>Storage &amp; privacy</h2><p>Local outputs, no upload pipeline.</p></div></div><div className="setting-info"><span>Default archives folder</span><strong>{appInfo?.outputFolder ?? "Documents / SHIFT / Archives"}</strong></div><button className="button button-secondary full-width" type="button" onClick={openOutputFolder}><FolderOpen size={16} /> Open destination folder</button><div className="privacy-list"><div><CheckCircle2 size={16} /><span>Core ZIP operation works locally</span></div><div><CheckCircle2 size={16} /><span>No account or cloud upload required</span></div><div><CheckCircle2 size={16} /><span>History remains in this session</span></div></div></section>
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><Activity size={18} /></span><div><h2>System status</h2><p>Dependency availability in this build.</p></div></div><div className="system-status-row"><span>Desktop bridge</span><strong className={isDesktop ? "system-ready" : "system-warn"}><i /> {isDesktop ? "Connected" : "Web preview only"}</strong></div><div className="system-status-row"><span>ZIP engine</span><strong className="system-ready"><i /> Rust · DEFLATE</strong></div><div className="system-status-row"><span>PDF engine</span><strong className="system-warn"><i /> Not connected</strong></div><div className="system-status-row"><span>Office conversion</span><strong className="system-warn"><i /> Not connected</strong></div><p className="settings-footnote">Statuses reflect integrations in this build, not guesses about installed software.</p></section>
        </div>
      </div>}

      {!(["home", "compress", "history", "settings"] as PageId[]).includes(page) && <div className="page planned-page"><div className="eyebrow"><Sparkles size={14} /> {PLANNED[page].eyebrow}</div><div className="planned-illustration"><div className="planned-ring ring-a" /><div className="planned-ring ring-b" /><div className="planned-center"><Layers3 size={28} /></div><span className="planned-orbit orbit-a"><FileText size={18} /></span><span className="planned-orbit orbit-b"><ShieldCheck size={18} /></span></div><h1>{PLANNED[page].title}</h1><p className="planned-description">{PLANNED[page].description}</p><div className="planned-status"><i /><div><strong>Not enabled in this build</strong><small>{PLANNED[page].engine}</small></div></div><p className="planned-footnote">No fake conversions. This operation will appear here after its engine and output tests pass.</p><button className="button button-primary" onClick={() => setPage("compress")} type="button">Use the ZIP tool instead <ArrowRight size={16} /></button></div>}
    </main>

    <button className="floating-wheel-trigger" type="button" aria-label="Quick action wheel" onClick={() => setWheelOpen((v) => !v)}><Plus size={22} /></button>
    {wheelOpen && <><button className="wheel-scrim" type="button" aria-label="Close quick action wheel" onClick={() => setWheelOpen(false)} /><div className="action-wheel" role="dialog" aria-modal="true" aria-labelledby="wheel-title"><div className="wheel-head"><span className="wheel-kicker">QUICK ACTIONS</span><button className="wheel-close" type="button" onClick={() => setWheelOpen(false)} aria-label="Close"><X size={16} /></button></div><h2 id="wheel-title">{files.length ? "What should happen next?" : "Start with a file."}</h2><p>{files.length ? "Only operations enabled for this selection are shown." : "Pick files first. This menu uses the same typed operation registry as the workspace."}</p>
      {availableOperations.length ? <div className="wheel-actions">{availableOperations.map((operation) => <button className="wheel-action" key={operation.id} onClick={() => selectOperation(operation)} type="button"><span className="wheel-action-icon"><Archive size={18} /></span><span><strong>{operation.title}</strong><small>{operation.outputLabel}</small></span><ArrowUpRight size={17} /></button>)}</div> : <button className="wheel-empty" type="button" onClick={() => { setWheelOpen(false); void chooseFiles(); }}><Upload size={19} /><span>Choose files</span><ArrowRight size={16} /></button>}
      <div className="wheel-footer"><Command size={13} /> ESC TO CLOSE <span>·</span> OPERATION REGISTRY</div></div></>}

    {toast && <div className={"toast toast-" + toast.kind} role="status" aria-live="polite">{toast.kind === "success" ? <CheckCircle2 size={19} /> : toast.kind === "error" ? <X size={18} /> : <Activity size={18} />}<span>{toast.message}</span><button type="button" aria-label="Dismiss message" onClick={() => setToast(null)}><X size={15} /></button></div>}
  </div>;
}

export default App;
