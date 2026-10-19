import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { AlertCircle, Archive, CheckCircle2, Command, FolderOpen, X } from "lucide-react";
import { formatBytes } from "../lib/format";

interface ZipResult {
  outputPath: string;
  inputBytes: number;
  archiveBytes: number;
  bytesSaved: number;
  filesArchived: number;
}

interface PhysicalPoint {
  x: number;
  y: number;
}

function leafName(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

function operationAt(position: PhysicalPoint): string | null {
  const scale = window.devicePixelRatio || 1;
  const element = document.elementFromPoint(position.x / scale, position.y / scale);
  return element?.closest<HTMLElement>("[data-operation]")?.dataset.operation ?? null;
}

export default function WheelOverlay() {
  const [activationKey, setActivationKey] = useState("F8");
  const [dragPaths, setDragPaths] = useState<string[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [isOverZip, setIsOverZip] = useState(false);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<ZipResult | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const hide = useCallback(async () => {
    try { await invoke("hide_wheel_overlay"); } catch { /* App may be shutting down. */ }
  }, []);

  const runZip = useCallback(async (paths: string[]) => {
    if (busy || paths.length === 0) return;
    setBusy(true);
    setNotice(null);
    setResult(null);
    try {
      const output = await invoke<ZipResult>("create_zip", { inputs: paths, archiveName: null });
      setResult(output);
      setDragPaths([]);
    } catch (error) {
      setNotice("ZIP failed: " + String(error));
    } finally {
      setBusy(false);
    }
  }, [busy]);

  useEffect(() => {
    document.documentElement.dataset.theme = localStorage.getItem("shift.theme") === "dark" ? "dark" : "light";
    document.documentElement.dataset.reducedMotion = localStorage.getItem("shift.reducedMotion") === "true" ? "true" : "false";
    let active = true;
    void invoke<string>("get_wheel_shortcut").then((key) => { if (active) setActivationKey(key); }).catch(() => {});
    return () => { active = false; };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter") {
        setDragPaths(event.payload.paths);
        setNotice(null);
        setResult(null);
        setIsDragging(true);
        setIsOverZip(operationAt(event.payload.position) === "create-zip");
      } else if (event.payload.type === "over") {
        setIsDragging(true);
        setIsOverZip(operationAt(event.payload.position) === "create-zip");
      } else if (event.payload.type === "leave") {
        setIsDragging(false);
        setIsOverZip(false);
      } else if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        const target = operationAt(event.payload.position);
        setDragPaths(paths);
        setIsDragging(false);
        setIsOverZip(false);
        if (target !== "create-zip") {
          setNotice("Drop the file onto the orange “Compress to ZIP” action.");
          return;
        }
        void runZip(paths);
      }
    }).then((stop) => {
      if (cancelled) stop();
      else unlisten = stop;
    }).catch((error: unknown) => setNotice("Windows file-drop support could not start: " + String(error)));
    return () => { cancelled = true; unlisten?.(); };
  }, [runZip]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void hide();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [hide]);

  const openFolder = useCallback(async () => {
    try { await invoke("open_output_folder"); }
    catch (error) { setNotice("Could not open the archive folder: " + String(error)); }
  }, []);

  return (
    <main className="shift-wheel-root" aria-label="SHIFT floating file wheel">
      <div className={"shift-wheel-shell" + (isDragging ? " is-dragging" : "") + (isOverZip ? " is-over-zip" : "")}>
        <div className="shift-wheel-orbit" aria-hidden="true" />
        <div className="shift-wheel-center">
          <span className="shift-wheel-mark"><span /></span>
          {result ? (
            <>
              <strong><CheckCircle2 size={17} /> ZIP ready</strong>
              <small>{formatBytes(result.archiveBytes)} · {result.filesArchived} entries</small>
            </>
          ) : busy ? (
            <>
              <span className="shift-wheel-spinner" />
              <strong>Creating ZIP…</strong>
              <small>Validating every entry</small>
            </>
          ) : isDragging ? (
            <>
              <strong>Drop to compress</strong>
              <small>{dragPaths.length === 1 ? leafName(dragPaths[0]) : dragPaths.length + " items selected"}</small>
            </>
          ) : (
            <>
              <strong>SHIFT wheel</strong>
              <small>Drag a file here</small>
            </>
          )}
        </div>
        <button
          type="button"
          className={"shift-wheel-action" + (isOverZip ? " drop-highlight" : "")}
          data-operation="create-zip"
          aria-label="Compress dropped files to ZIP"
          onClick={() => setNotice("Drag a file from Explorer and drop it on this action.")}
          disabled={busy}
        >
          <span className="shift-wheel-action-icon"><Archive size={23} /></span>
          <strong>Compress to ZIP</strong>
          <small>{isOverZip ? "RELEASE TO START" : "DROP FILES HERE"}</small>
        </button>
        <button className="shift-wheel-close" type="button" onClick={() => void hide()} aria-label="Close SHIFT wheel">
          <X size={15} />
        </button>
        <div className="shift-wheel-foot"><Command size={13} /> HOLD {activationKey} · ESC TO CLOSE</div>
        {notice && <div className="shift-wheel-notice" role="status"><AlertCircle size={15} /><span>{notice}</span><button type="button" onClick={() => setNotice(null)} aria-label="Dismiss message"><X size={13} /></button></div>}
        {result && <div className="shift-wheel-result" role="status">
          <strong><CheckCircle2 size={15} /> Archive verified</strong>
          <span>{leafName(result.outputPath)}</span>
          <small>{formatBytes(result.inputBytes)} → {formatBytes(result.archiveBytes)}</small>
          <button type="button" onClick={() => void openFolder()}><FolderOpen size={14} /> Show output folder</button>
        </div>}
      </div>
    </main>
  );
}
