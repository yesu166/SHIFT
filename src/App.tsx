import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Activity, Archive, ArrowRight, ArrowUpRight, Check, CheckCircle2,
  ChevronRight, Clock3, Command, FileArchive, FileClock, FileText, FolderOpen, HardDrive,
  Home, LockKeyhole, Moon, MoreHorizontal, Plus, Settings, ShieldCheck, Sparkles,
  Sun, Trash2, Upload, X, Zap, type LucideIcon,
} from "lucide-react";
import { formatBytes, formatSignedBytes } from "./lib/format";
import { getAvailableOperations, NO_OPTIONAL_ENGINES, type EngineAvailability } from "./lib/operations";

type PageId = "home" | "compress" | "history" | "settings";
type ThemeMode = "light" | "dark";
type GestureMode = "convert" | "tools" | "actions";
type GestureShortcutId = "shift" | "ctrl" | "alt" | "ctrlShift" | "altShift" | "ctrlAlt" | "ctrlAltShift";
interface GestureShortcuts { convert: GestureShortcutId; tools: GestureShortcutId; actions: GestureShortcutId }
const MIN_WHEEL_SIZE = 360;
const MAX_WHEEL_SIZE = 720;
const GESTURE_SHORTCUT_OPTIONS: { value: GestureShortcutId; label: string }[] = [
  { value: "shift", label: "Shift" },
  { value: "ctrl", label: "Ctrl" },
  { value: "alt", label: "Alt" },
  { value: "ctrlShift", label: "Ctrl + Shift" },
  { value: "altShift", label: "Alt + Shift" },
  { value: "ctrlAlt", label: "Ctrl + Alt" },
  { value: "ctrlAltShift", label: "Ctrl + Alt + Shift" },
];
function shortcutLabel(shortcut: GestureShortcutId): string {
  return GESTURE_SHORTCUT_OPTIONS.find((option) => option.value === shortcut)?.label ?? "Shift";
}
interface QueuedFile { id: string; name: string; path?: string; size?: number }
interface ZipResult { outputPath: string; inputBytes: number; archiveBytes: number; bytesSaved: number; filesArchived: number }
interface AppInfo { outputFolder: string; appVersion: string }
interface JobRecord extends ZipResult { id: string; completedAt: string; status: "done" }
interface NavItem { id: PageId; label: string; icon: LucideIcon }

const NAV_ITEMS: NavItem[] = [
  { id: "home", label: "Home", icon: Home },
  { id: "compress", label: "Compress", icon: Archive },
  { id: "history", label: "History", icon: FileClock },
  { id: "settings", label: "Settings", icon: Settings },
];

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
  const [engineStatus, setEngineStatus] = useState<EngineAvailability>(NO_OPTIONAL_ENGINES);
  const [gestureShortcuts, setGestureShortcuts] = useState<GestureShortcuts>({
    convert: "shift",
    tools: "altShift",
    actions: "ctrlShift",
  });
  const [savingShortcuts, setSavingShortcuts] = useState(false);
  const [shortcutStatus, setShortcutStatus] = useState<string | null>(null);
  const [wheelSize, setWheelSize] = useState(520);
  const [draftWheelSize, setDraftWheelSize] = useState(520);
  const [savingWheelSize, setSavingWheelSize] = useState(false);
  const [wheelSizeStatus, setWheelSizeStatus] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; kind: "success" | "error" | "info" } | null>(null);
  const [theme, setTheme] = useState<ThemeMode>(() => localStorage.getItem("shift.theme") === "dark" ? "dark" : "light");
  const [reducedMotion, setReducedMotion] = useState(() => localStorage.getItem("shift.reducedMotion") === "true");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const isDesktop = isTauri();

  const availableOperations = useMemo(() => getAvailableOperations(files.map((file) => file.name), engineStatus), [files, engineStatus]);
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
      void invoke<EngineAvailability>("get_engine_status")
        .then((status) => { if (active) setEngineStatus(status); })
        .catch((error: unknown) => { if (active) setToast({ message: "Unable to detect optional engines: " + String(error), kind: "error" }); });
      void invoke<GestureShortcuts>("get_gesture_shortcuts")
        .then((shortcuts) => { if (active) setGestureShortcuts(shortcuts); })
        .catch((error: unknown) => { if (active) setShortcutStatus("Could not load shortcuts: " + String(error)); });
      void invoke<number>("get_wheel_size")
        .then((size) => { if (active) { setWheelSize(size); setDraftWheelSize(size); } })
        .catch((error: unknown) => { if (active) setWheelSizeStatus("Could not load wheel size: " + String(error)); });
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
          const droppedPaths = event.payload.paths;
          setFiles((current) => uniqueFiles(current, droppedPaths.map((path) => ({ id: makeId(), name: leafName(path), path }))));
          setPage("compress");
        }
      }),
    ).then((stop) => { if (cancelled) stop(); else unlisten = stop; })
      .catch((error: unknown) => setToast({ message: "File drop support could not start: " + String(error), kind: "error" }));
    return () => { cancelled = true; unlisten?.(); };
  }, [isDesktop]);

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

  const saveGestureShortcut = useCallback(async (mode: GestureMode, value: GestureShortcutId) => {
    const previous = gestureShortcuts;
    const next = { ...gestureShortcuts, [mode]: value };
    setGestureShortcuts(next);
    setSavingShortcuts(true);
    setShortcutStatus("Saving shortcuts…");
    try {
      const saved = await invoke<GestureShortcuts>("update_gesture_shortcuts", { shortcuts: next });
      setGestureShortcuts(saved);
      setShortcutStatus("Shortcuts saved.");
    } catch (error) {
      setGestureShortcuts(previous);
      setShortcutStatus(String(error));
    } finally {
      setSavingShortcuts(false);
    }
  }, [gestureShortcuts]);

  const saveWheelSize = useCallback(async () => {
    setSavingWheelSize(true);
    setWheelSizeStatus("Saving wheel size…");
    try {
      const saved = await invoke<number>("update_wheel_size", { size: draftWheelSize });
      setWheelSize(saved);
      setDraftWheelSize(saved);
      setWheelSizeStatus("Wheel size saved.");
    } catch (error) {
      setDraftWheelSize(wheelSize);
      setWheelSizeStatus(String(error));
    } finally {
      setSavingWheelSize(false);
    }
  }, [draftWheelSize, wheelSize]);

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

  const chooseFolders = useCallback(async () => {
    if (!isDesktop) {
      setToast({ message: "Folder selection is available in the Windows desktop build.", kind: "info" });
      return;
    }
    try {
      const selected = await open({
        multiple: true,
        directory: true,
        title: "Choose folders for your SHIFT archive",
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      setFiles((current) => uniqueFiles(current, paths.map((path) => ({
        id: makeId(),
        name: leafName(path),
        path,
      }))));
      setPage("compress");
    } catch (error) {
      setToast({ message: "The folder picker failed: " + String(error), kind: "error" });
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
      const job: JobRecord = { ...result, id: makeId(), completedAt: new Date().toISOString(), status: "done" };
      setJobs((current) => [job, ...current].slice(0, 30));
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



  const currentTitle = NAV_ITEMS.find((item) => item.id === page)?.label ?? "Home";

  return <div className="app-shell">
    <aside className="sidebar">
      <button className="brand-lockup" type="button" onClick={() => setPage("home")} aria-label="SHIFT home">
        <span className="brand-mark"><span /></span><span className="brand-name">shift<span>.</span><small>DOCUMENT TOOLBOX</small></span>
      </button>
      <div className="workspace-label">WORKSPACE <span>01</span></div>
      <nav className="primary-nav" aria-label="Main navigation">
        {NAV_ITEMS.slice(0, 2).map((item) => {
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
      <header className="topbar"><div className="breadcrumbs"><span>SHIFT</span><ChevronRight size={14} /><strong>{currentTitle}</strong></div><div className="topbar-actions"><div className={"local-pill" + (isDesktop ? " connected" : "")}><i />{isDesktop ? "LOCAL MODE" : "PREVIEW MODE"}</div><div className="shortcut-hint">{shortcutLabel(gestureShortcuts.convert)} · {shortcutLabel(gestureShortcuts.tools)} · {shortcutLabel(gestureShortcuts.actions)}</div></div></header>

      {page === "home" && <div className="page page-home">
        <div className="page-intro"><div><div className="eyebrow"><Sparkles size={14} /> WINDOWS FILE TOOLS</div><h1>Convert and compress<br /><span>at the cursor.</span></h1><p className="intro-copy">Drag a file from your Desktop or File Explorer, then hold <strong>{shortcutLabel(gestureShortcuts.convert)}</strong> to convert, <strong>{shortcutLabel(gestureShortcuts.tools)}</strong> for tools, or <strong>{shortcutLabel(gestureShortcuts.actions)}</strong> for file actions.</p></div>
          <div className="intro-orbit" aria-hidden="true"><div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="orbit-center"><span className="brand-mark"><span /></span></div><span className="orbit-file orbit-a"><FileText size={19} /></span><span className="orbit-file orbit-b"><FileArchive size={19} /></span><span className="orbit-file orbit-c"><CheckCircle2 size={19} /></span></div>
        </div>
        <div className={"drop-card" + (isDragging ? " drag-active" : "")} onDragEnter={(e) => { e.preventDefault(); setIsDragging(true); }} onDragOver={(e) => e.preventDefault()} onDragLeave={(e) => { if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setIsDragging(false); }} onDrop={(e) => { e.preventDefault(); setIsDragging(false); addBrowserFiles(e.dataTransfer.files); }}>
          <input ref={fileInputRef} className="hidden-file-input" type="file" multiple onChange={(e) => { if (e.target.files) addBrowserFiles(e.target.files); e.currentTarget.value = ""; }} />
          <div className="drop-inner"><div className="upload-illustration"><div className="upload-sheet"><FileText size={23} /><span /></div><div className="upload-arrow"><Upload size={18} /></div></div>
            <div className="drop-copy"><h2>Drop files to get moving</h2><p>Convert supported image formats, compress JPEGs, or bundle any files and folders into a ZIP.</p><div className="drop-actions"><button className="button button-primary" type="button" onClick={chooseFiles}>Choose files <ArrowRight size={16} /></button><button className="button button-secondary" type="button" onClick={chooseFolders}><FolderOpen size={15} /> Choose folders</button><span>or drag them here</span></div></div>
            <div className="drop-format-row"><span>PNG</span><span>JPG</span><span>WEBP</span><span>TIFF</span><span>+ ANY FILE → ZIP</span></div>
          </div>
        </div>
        <section className="wheel-instructions-card">
          <div className="wheel-instructions-icon"><Command size={20} /></div>
          <div className="wheel-instructions-copy">
            <span className="eyebrow">GLOBAL QUICK ACTION</span>
            <h2>Use SHIFT over any app</h2>
            <p>Start dragging a file, then tap <strong>Shift</strong> for conversions, <strong>Alt+Shift</strong> for tools, or <strong>Ctrl+Shift</strong> for file actions. The wheel stays open until you drop or cancel.</p>
          </div>
          <button className="button button-secondary" type="button" onClick={() => setPage("settings")}>Change key</button>
        </section>
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
          <section className="workspace-card"><div className="card-topline"><span className="step-number">01</span><div><h2>Add your files</h2><p>Mix formats freely. Folders and nested files are supported.</p></div><div className="card-tools"><button className="icon-button subtle" type="button" onClick={chooseFiles} title="Add files" aria-label="Add files"><Plus size={19} /></button><button className="icon-button subtle" type="button" onClick={chooseFolders} title="Add folders" aria-label="Add folders"><FolderOpen size={16} /></button></div></div>
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

      {page === "settings" && <div className="page"><div className="page-title-row"><div><div className="eyebrow"><Settings size={14} /> SETTINGS</div><h1>Settings</h1><p>Configure shortcuts, wheel size, appearance, and output locations.</p></div></div>
        <div className="settings-layout">
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><Command size={18} /></span><div><h2>Drag shortcuts</h2><p>Change the wheel mode while dragging a file.</p></div></div>
            <div className="gesture-shortcut-list">
              {([
                { mode: "convert", title: "Convert", detail: "Show formats supported by the dragged files." },
                { mode: "tools", title: "Tools", detail: "Compress or package supported files." },
                { mode: "actions", title: "File actions", detail: "Copy paths or open the output location." },
              ] as { mode: GestureMode; title: string; detail: string }[]).map(({ mode, title, detail }) => (
                <label className="gesture-shortcut-setting" key={mode}>
                  <span className="gesture-shortcut-copy"><strong>{title}</strong><small>{detail}</small></span>
                  <select
                    className="shortcut-select"
                    aria-label={title + " drag shortcut"}
                    value={gestureShortcuts[mode]}
                    disabled={!isDesktop || savingShortcuts}
                    onChange={(event) => void saveGestureShortcut(mode, event.target.value as GestureShortcutId)}
                  >
                    {GESTURE_SHORTCUT_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>{option.label}</option>
                    ))}
                  </select>
                </label>
              ))}
            </div>
            {shortcutStatus && <p className="settings-footnote" role="status">{shortcutStatus}</p>}
            <p className="settings-footnote">Drag a file from the Desktop or File Explorer, hold your chosen modifier combo, then move onto a wheel action and release. Outputs are created beside the original file with a “(SHIFT converted)” or operation-specific suffix. SHIFT never replaces the source file. Press Escape to cancel.</p>
            <div className="privacy-list"><div><CheckCircle2 size={16} /><span>Native floating wheel over Desktop and Explorer</span></div><div><CheckCircle2 size={16} /><span>Runs locally without file uploads</span></div></div>
          </section>
          <section className="workspace-card settings-card wheel-size-card">
            <div className="settings-section-heading"><span className="settings-icon"><Command size={18} /></span><div><h2>Wheel size</h2><p>Choose how large the wheel appears around the pointer.</p></div></div>
            <div className="wheel-size-heading"><label htmlFor="wheel-size-slider">Wheel diameter</label><strong>{draftWheelSize}px</strong></div>
            <input
              id="wheel-size-slider"
              className="wheel-size-slider"
              type="range"
              min={MIN_WHEEL_SIZE}
              max={MAX_WHEEL_SIZE}
              step={20}
              value={draftWheelSize}
              disabled={!isDesktop || savingWheelSize}
              onChange={(event) => { setDraftWheelSize(Number(event.target.value)); setWheelSizeStatus(null); }}
              aria-describedby="wheel-size-help"
            />
            <div className="wheel-size-range-labels"><span>Compact · {MIN_WHEEL_SIZE}px</span><span>Large · {MAX_WHEEL_SIZE}px</span></div>
            <p id="wheel-size-help" className="settings-footnote">The new size applies the next time the wheel opens. Choose a size that keeps the actions easy to target without covering too much of the screen.</p>
            {wheelSizeStatus && <p className="settings-footnote" role="status">{wheelSizeStatus}</p>}
            <button className="button button-primary full-width" type="button" onClick={() => void saveWheelSize()} disabled={!isDesktop || savingWheelSize || draftWheelSize === wheelSize}>
              {savingWheelSize ? "Saving…" : "Save wheel size"}
            </button>
          </section>
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><Sun size={18} /></span><div><h2>Appearance</h2><p>Set the theme and animation preferences.</p></div></div>
            <div className="theme-picker" role="group" aria-label="Color theme"><button className={"theme-choice" + (theme === "light" ? " selected" : "")} type="button" onClick={() => setTheme("light")}><span className="theme-swatch theme-light"><i /><i /><i /></span><span>Light</span>{theme === "light" && <Check size={16} />}</button><button className={"theme-choice" + (theme === "dark" ? " selected" : "")} type="button" onClick={() => setTheme("dark")}><span className="theme-swatch theme-dark"><i /><i /><i /></span><span>Dark</span>{theme === "dark" && <Check size={16} />}</button></div>
            <div className="settings-divider" /><div className="settings-row"><span className="settings-icon">{reducedMotion ? <Moon size={18} /> : <Zap size={18} />}</span><span className="settings-row-copy"><strong>Reduce motion</strong><small>Limit interface transitions and movement.</small></span><button className={"toggle" + (reducedMotion ? " on" : "")} type="button" role="switch" aria-checked={reducedMotion} onClick={() => setReducedMotion(!reducedMotion)} aria-label="Reduce motion"><span /></button></div>
          </section>
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><HardDrive size={18} /></span><div><h2>Storage</h2><p>Choose and inspect local output locations.</p></div></div><div className="setting-info"><span>Default archive folder</span><strong>{appInfo?.outputFolder ?? "Documents / SHIFT / Archives"}</strong></div><button className="button button-secondary full-width" type="button" onClick={openOutputFolder}><FolderOpen size={16} /> Open destination folder</button><div className="privacy-list"><div><CheckCircle2 size={16} /><span>Core ZIP operation works locally</span></div><div><CheckCircle2 size={16} /><span>No account or cloud upload required</span></div><div><CheckCircle2 size={16} /><span>History remains in this session</span></div></div></section>
          <section className="workspace-card settings-card"><div className="settings-section-heading"><span className="settings-icon"><Activity size={18} /></span><div><h2>System status</h2><p>Check which local tools are available.</p></div></div><div className="system-status-row"><span>Desktop bridge</span><strong className={isDesktop ? "system-ready" : "system-warn"}><i /> {isDesktop ? "Connected" : "Web preview only"}</strong></div><div className="system-status-row"><span>ZIP engine</span><strong className="system-ready"><i /> Rust · DEFLATE</strong></div><div className="system-status-row"><span>Image conversion</span><strong className="system-ready"><i /> Rust · 5 formats</strong></div><div className="system-status-row"><span>JPEG compression</span><strong className="system-ready"><i /> Quality 78</strong></div><div className="system-status-row"><span>PDF tools (qpdf)</span><strong className={engineStatus.qpdf ? "system-ready" : "system-warn"}><i /> {engineStatus.qpdf ? "Detected" : "Install qpdf"}</strong></div>
          <div className="system-status-row"><span>PDF render/text (Poppler)</span><strong className={engineStatus.poppler ? "system-ready" : "system-warn"}><i /> {engineStatus.poppler ? "Detected" : "Install Poppler"}</strong></div>
          <div className="system-status-row"><span>SVG export (Inkscape)</span><strong className={engineStatus.inkscape ? "system-ready" : "system-warn"}><i /> {engineStatus.inkscape ? "Inkscape detected" : "Install Inkscape"}</strong></div>
          <div className="system-status-row"><span>Audio/video + advanced images</span><strong className={engineStatus.ffmpeg ? "system-ready" : "system-warn"}><i /> {engineStatus.ffmpeg ? "FFmpeg detected" : "Install FFmpeg"}</strong></div>
          <div className="system-status-row"><span>Documents, slides, sheets</span><strong className={engineStatus.libreOffice ? "system-ready" : "system-warn"}><i /> {engineStatus.libreOffice ? "LibreOffice detected" : "Install LibreOffice"}</strong></div>
          <div className="system-status-row"><span>RAR / 7Z archive conversion</span><strong className={engineStatus.sevenZip ? "system-ready" : "system-warn"}><i /> {engineStatus.sevenZip ? "7-Zip detected" : "Install 7-Zip"}</strong></div>
          <div className="system-status-row"><span>TAR / TAR.GZ / GZIP</span><strong className="system-ready"><i /> Built in</strong></div>
          <div className="system-status-row"><span>Subtitles SRT / VTT / TXT</span><strong className="system-ready"><i /> Built in</strong></div>
          <p className="settings-footnote">SHIFT detects optional tools on startup. Install the desired tool, add it to PATH, then fully restart SHIFT. The wheel only offers actions when their engine is detected.</p></section>
        </div>
      </div>}


    </main>

    {toast && <div className={"toast toast-" + toast.kind} role="status" aria-live="polite">{toast.kind === "success" ? <CheckCircle2 size={19} /> : toast.kind === "error" ? <X size={18} /> : <Activity size={18} />}<span>{toast.message}</span><button type="button" aria-label="Dismiss message" onClick={() => setToast(null)}><X size={15} /></button></div>}
  </div>;
}

export default App;
