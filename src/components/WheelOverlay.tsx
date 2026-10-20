import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  Archive, ArrowRightLeft, CheckCircle2, Clipboard, FileImage, FolderOpen,
  ImageDown, Layers, X, Zap,
} from "lucide-react";
import { formatBytes } from "../lib/format";
import { getRadialPosition } from "../lib/radial";
import { extensionOf, getWheelOperations, NO_OPTIONAL_ENGINES, type EngineAvailability } from "../lib/operations";

type WheelMode = "convert" | "tools" | "actions";
type GestureShortcutId = "shift" | "ctrl" | "alt" | "ctrlShift" | "altShift" | "ctrlAlt" | "ctrlAltShift";
interface GestureShortcuts { convert: GestureShortcutId; tools: GestureShortcutId; actions: GestureShortcutId }
const DEFAULT_GESTURE_SHORTCUTS: GestureShortcuts = { convert: "shift", tools: "altShift", actions: "ctrlShift" };
const SHORTCUT_LABELS: Record<GestureShortcutId, string> = {
  shift: "Shift", ctrl: "Ctrl", alt: "Alt", ctrlShift: "Ctrl + Shift",
  altShift: "Alt + Shift", ctrlAlt: "Ctrl + Alt", ctrlAltShift: "Ctrl + Alt + Shift",
};

interface ZipResult {
  outputPath: string;
  inputBytes: number;
  archiveBytes: number;
  bytesSaved: number;
  filesArchived: number;
}
interface ImageResult {
  inputPath: string;
  outputPath: string;
  inputBytes: number;
  outputBytes: number;
  bytesSaved: number;
  detail?: string;
}
interface WheelAction {
  id: string;
  title: string;
  detail: string;
  kind: "convert" | "compress" | "archive" | "tar" | "gzip" | "copy" | "folder" | "engine";
  target?: string;
}
interface JobSummary {
  title: string;
  outputs: string[];
  inputBytes: number;
  outputBytes: number;
  detail: string;
}
interface PhysicalPoint { x: number; y: number }

function leafName(path: string): string { return path.split(/[\\/]/).pop() || path; }
function titleForMode(mode: WheelMode): string {
  return mode === "convert" ? "Convert" : mode === "tools" ? "Tools" : "File actions";
}
function hintForMode(mode: WheelMode, shortcuts: GestureShortcuts): string {
  return SHORTCUT_LABELS[shortcuts[mode]].toUpperCase() + " · " + titleForMode(mode).toUpperCase();
}
function alternateModeHint(mode: WheelMode, shortcuts: GestureShortcuts): string {
  return (["convert", "tools", "actions"] as WheelMode[])
    .filter((candidate) => candidate !== mode)
    .map((candidate) => SHORTCUT_LABELS[shortcuts[candidate]] + ": " + titleForMode(candidate))
    .join("  ·  ");
}
function iconFor(action: WheelAction) {
  if (action.kind === "archive" || action.kind === "tar" || action.kind === "gzip") return Archive;
  if (action.kind === "compress") return ImageDown;
  if (action.kind === "copy") return Clipboard;
  if (action.kind === "folder") return FolderOpen;
  if (action.kind === "convert") return FileImage;
  return ArrowRightLeft;
}
function buildActions(paths: string[], mode: WheelMode, engines: EngineAvailability): WheelAction[] {
  return getWheelOperations(paths.map(leafName), mode, engines).map((operation) => {
    const id = operation.id;
    const target = operation.targetFormat;
    const kind: WheelAction["kind"] = id.startsWith("convert-") ? "convert"
      : id === "compress-jpeg" ? "compress"
      : id === "create-zip" ? "archive"
      : id === "create-tar" || id === "create-targz" ? "tar"
      : id === "create-gzip" ? "gzip"
      : id === "copy-paths" ? "copy"
      : id === "open-folder" ? "folder" : "engine";
    return {
      id,
      title: operation.title,
      detail: id === "compress-jpeg" ? "Quality 78"
        : id === "create-zip" ? "Validated ZIP"
        : id === "create-tar" ? "Bundle files"
        : id === "create-targz" ? "Compress archive"
        : id === "create-gzip" ? "Single file"
        : id === "pdf-optimize" ? "Reduce PDF size"
        : id === "pdf-merge" ? "Combine PDFs"
        : id.startsWith("media-compress-") ? "Re-encode locally"
        : operation.requiredEngine ? "Uses " + operation.requiredEngine
        : target ? "Convert to " + target.toUpperCase()
        : id === "copy-paths" ? "To clipboard"
        : id === "open-folder" ? "Open destination" : operation.description,
      kind,
      ...(target ? { target } : {}),
    };
  });
}

function operationAt(position: PhysicalPoint): string | null {
  const scale = window.devicePixelRatio || 1;
  const element = document.elementFromPoint(position.x / scale, position.y / scale);
  return element?.closest<HTMLElement>("[data-operation]")?.dataset.operation ?? null;
}

export default function WheelOverlay() {
  const [mode, setMode] = useState<WheelMode>("convert");
  const [gestureShortcuts, setGestureShortcuts] = useState<GestureShortcuts>(DEFAULT_GESTURE_SHORTCUTS);
  const [wheelOpenId, setWheelOpenId] = useState(0);
  const [dragPaths, setDragPaths] = useState<string[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [hoveredAction, setHoveredAction] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [summary, setSummary] = useState<JobSummary | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [engineStatus, setEngineStatus] = useState<EngineAvailability>(NO_OPTIONAL_ENGINES);
  const engineStatusRef = useRef<EngineAvailability>(NO_OPTIONAL_ENGINES);
  const busyRef = useRef(false);
  const handledDropRef = useRef(false);
  const modeRef = useRef<WheelMode>("convert");
  const actions = useMemo(() => buildActions(dragPaths, mode, engineStatus), [dragPaths, mode, engineStatus]);

  useEffect(() => { busyRef.current = busy; }, [busy]);
  useEffect(() => { modeRef.current = mode; }, [mode]);
  useEffect(() => { engineStatusRef.current = engineStatus; }, [engineStatus]);

  const hide = useCallback(async () => {
    try { await invoke("hide_wheel_overlay"); } catch { /* App may already be closing. */ }
  }, []);

  const executeAction = useCallback(async (actionId: string, paths: string[]) => {
    if (!paths.length) {
      setNotice("Start dragging a file from File Explorer first.");
      return;
    }
    if (busyRef.current) return;
    const action = buildActions(paths, modeRef.current, engineStatusRef.current).find((item) => item.id === actionId);
    if (!action) {
      setNotice("That action is not available for this selection.");
      return;
    }

    busyRef.current = true;
    setBusy(true);
    setNotice(null);
    setSummary(null);
    try {
      if (action.kind === "archive") {
        const output = await invoke<ZipResult>("create_zip", { inputs: paths, archiveName: null });
        setSummary({
          title: "ZIP verified",
          outputs: [output.outputPath],
          inputBytes: output.inputBytes,
          outputBytes: output.archiveBytes,
          detail: output.filesArchived + " archive entries",
        });
      } else if (action.kind === "convert" && action.target) {
        const results = await invoke<ImageResult[]>("convert_images_to_format", {
          inputs: paths,
          targetFormat: action.target,
        });
        setSummary({
          title: "Conversion complete",
          outputs: results.map((result) => result.outputPath),
          inputBytes: results.reduce((sum, result) => sum + result.inputBytes, 0),
          outputBytes: results.reduce((sum, result) => sum + result.outputBytes, 0),
          detail: results.length + " image" + (results.length === 1 ? "" : "s") + " converted to " + action.target.toUpperCase(),
        });
      } else if (action.kind === "tar" || action.kind === "gzip" || action.kind === "engine") {
        const operationId = action.id.startsWith("ffmpeg-image-convert-") ? "media-convert"
          : action.id.startsWith("media-audio-") || action.id.startsWith("media-video-") ? "media-convert"
          : action.id.startsWith("extract-audio-") ? "extract-audio"
          : action.id.startsWith("video-frame-") ? "media-convert"
          : action.id.startsWith("media-compress-") ? "media-compress"
          : action.id.startsWith("svg-to-") ? "svg-convert"
          : action.id === "image-to-pdf" || action.id.startsWith("office-convert-") || action.id.startsWith("presentation-convert-") || action.id.startsWith("spreadsheet-convert-") || action.id.startsWith("pdf-office-") ? "office-convert"
          : action.id.startsWith("pdf-to-image-") ? "pdf-to-image"
          : action.id.startsWith("pdf-to-text-") ? "pdf-to-text"
          : action.id.startsWith("subtitle-") ? "subtitle-convert"
          : action.id === "archive-to-zip" ? "rar-to-zip" : action.id;
        const results = await invoke<ImageResult[]>("execute_engine_operation", {
          inputs: paths,
          operationId,
          targetFormat: action.target ?? null,
        });
        const uniqueInputs = Array.from(new Map(results.map((result) => [result.inputPath, result.inputBytes])).values());
        setSummary({
          title: action.title + " complete",
          outputs: results.map((result) => result.outputPath),
          inputBytes: uniqueInputs.reduce((sum, value) => sum + value, 0),
          outputBytes: results.reduce((sum, result) => sum + result.outputBytes, 0),
          detail: results.length + " output" + (results.length === 1 ? "" : "s") + (results[0]?.detail ? " · " + results[0].detail : ""),
        });
      } else if (action.kind === "compress") {
        const results = await invoke<ImageResult[]>("compress_jpeg_images", { inputs: paths });
        setSummary({
          title: "Compression complete",
          outputs: results.map((result) => result.outputPath),
          inputBytes: results.reduce((sum, result) => sum + result.inputBytes, 0),
          outputBytes: results.reduce((sum, result) => sum + result.outputBytes, 0),
          detail: results.length + " JPEG image" + (results.length === 1 ? "" : "s") + " re-encoded",
        });
      } else if (action.kind === "copy") {
        await navigator.clipboard.writeText(paths.join("\r\n"));
        setSummary({ title: "Paths copied", outputs: [], inputBytes: 0, outputBytes: 0, detail: paths.length + " local path" + (paths.length === 1 ? "" : "s") + " copied" });
      } else if (action.kind === "folder") {
        await invoke("open_output_folder");
        setSummary({ title: "Output folder opened", outputs: [], inputBytes: 0, outputBytes: 0, detail: "SHIFT's local destination folder" });
      }
    } catch (error) {
      setNotice(String(error));
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void invoke<EngineAvailability>("get_engine_status").then((status) => setEngineStatus(status)).catch(() => {});
    void invoke<GestureShortcuts>("get_gesture_shortcuts").then(setGestureShortcuts).catch(() => {});
    document.documentElement.dataset.theme = localStorage.getItem("shift.theme") === "dark" ? "dark" : "light";
    document.documentElement.dataset.reducedMotion = localStorage.getItem("shift.reducedMotion") === "true" ? "true" : "false";

    let cancelled = false;
    const unlisteners: (() => void)[] = [];
    void listen<string>("wheel-mode", (event) => {
      const nextMode = event.payload;
      if (nextMode === "convert" || nextMode === "tools" || nextMode === "actions") {
        modeRef.current = nextMode;
        setMode(nextMode);
        handledDropRef.current = false;
        setSummary(null);
        setNotice(null);
      }
    }).then((stop) => { if (cancelled) stop(); else unlisteners.push(stop); }).catch(() => {});
    void listen("wheel-opened", () => {
      setWheelOpenId((current) => current + 1);
    }).then((stop) => { if (cancelled) stop(); else unlisteners.push(stop); }).catch(() => {});
    void listen("wheel-gesture-end", () => {
      // Give the WebView's native drop event time to start the operation before closing.
      window.setTimeout(() => { if (!busyRef.current) void hide(); }, 260);
    }).then((stop) => { if (cancelled) stop(); else unlisteners.push(stop); }).catch(() => {});
    return () => { cancelled = true; unlisteners.forEach((stop) => stop()); };
  }, [hide]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter") {
        setDragPaths(event.payload.paths);
        handledDropRef.current = false;
        setNotice(null);
        setSummary(null);
        setIsDragging(true);
        setHoveredAction(operationAt(event.payload.position));
      } else if (event.payload.type === "over") {
        setIsDragging(true);
        setHoveredAction(operationAt(event.payload.position));
      } else if (event.payload.type === "leave") {
        setIsDragging(false);
        setHoveredAction(null);
      } else if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        const actionId = operationAt(event.payload.position);
        setDragPaths(paths);
        setIsDragging(false);
        setHoveredAction(null);
        if (!actionId) {
          setNotice("Drop on one of the round actions to run it.");
          return;
        }
        handledDropRef.current = true;
        void executeAction(actionId, paths);
      }
    }).then((stop) => { if (cancelled) stop(); else unlisten = stop; })
      .catch((error: unknown) => setNotice("Windows drag-and-drop support could not start: " + String(error)));
    return () => { cancelled = true; unlisten?.(); };
  }, [executeAction]);

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

  const selectedTypes = Array.from(new Set(
    dragPaths.map((path) => extensionOf(leafName(path))).filter(Boolean),
  ));
  const selectedTypeLabel = selectedTypes.length ? selectedTypes.join(", ") : "this selection";
  const actionHint = !dragPaths.length
    ? "Waiting for a file drag"
    : actions.length
      ? "Release over an action to run it"
      : "No " + titleForMode(mode).toLowerCase() + " options for " + selectedTypeLabel;
  const modeHelp = alternateModeHint(mode, gestureShortcuts);

  return (
    <main className="shift-wheel-root" aria-label="SHIFT contextual file wheel">
      <div className={"shift-wheel-shell wheel-open-" + (wheelOpenId % 2) + (isDragging ? " is-dragging" : "") + (hoveredAction ? " is-over-zip" : "") + (summary ? " has-result" : "")}>
        <div className="shift-wheel-orbit" aria-hidden="true" />
        <div className="shift-wheel-center">
          <span className="shift-wheel-mark"><span /></span>
          {summary ? (
            <>
              <strong><CheckCircle2 size={17} /> {summary.title}</strong>
              <small>{summary.detail}</small>
              {summary.outputs.length > 0 && <small className="wheel-output-name">{leafName(summary.outputs[0])}{summary.outputs.length > 1 ? " +" + (summary.outputs.length - 1) + " more" : ""}</small>}
              {summary.outputs.length > 0 && <small>{formatBytes(summary.inputBytes)} → {formatBytes(summary.outputBytes)}</small>}
            </>
          ) : busy ? (
            <>
              <span className="shift-wheel-spinner" />
              <strong>Working locally…</strong>
              <small>Original files are kept intact</small>
            </>
          ) : (
            <>
              <span className="wheel-mode-label">{titleForMode(mode)}</span>
              <strong>{isDragging ? "Drop to run" : "SHIFT"}</strong>
              <small>{isDragging ? dragPaths.length + " file" + (dragPaths.length === 1 ? "" : "s") + " detected" : actionHint}</small>
            </>
          )}
        </div>
        {!summary && actions.map((action, index) => {
          const point = getRadialPosition(index, actions.length, 37);
          const Icon = iconFor(action);
          return (
            <button
              key={wheelOpenId + "-" + action.id}
              type="button"
              className={"shift-wheel-action" + (hoveredAction === action.id ? " drop-highlight" : "")}
              data-operation={action.id}
              aria-label={action.title}
              title={action.detail}
              style={{ left: point.x + "%", top: point.y + "%", animationDelay: Math.min(index, 8) * 18 + "ms" }}
              onClick={() => void executeAction(action.id, dragPaths)}
              disabled={busy}
            >
              <span className="shift-wheel-action-icon"><Icon size={20} /></span>
              <strong>{action.title}</strong>
              <small>{action.detail}</small>
            </button>
          );
        })}
        {!actions.length && !summary && !busy && (
          <div className="shift-wheel-no-actions">
            <Layers size={15} />
            <span>{actionHint}</span>
            {dragPaths.length > 0 && <small>{modeHelp}</small>}
          </div>
        )}
        <button className="shift-wheel-close" type="button" onClick={() => void hide()} aria-label="Close SHIFT wheel"><X size={15} /></button>
        <div className="shift-wheel-foot"><Zap size={12} /> {hintForMode(mode, gestureShortcuts)} · ESC TO CLOSE</div>
        {notice && <div className="shift-wheel-notice" role="status"><span>{notice}</span><button type="button" onClick={() => setNotice(null)} aria-label="Dismiss message"><X size={13} /></button></div>}
        {summary && (
          <div className="shift-wheel-result" role="status">
            <strong><CheckCircle2 size={15} /> Finished safely</strong>
            <span>{summary.outputs[0] ? leafName(summary.outputs[0]) : summary.detail}</span>
            <small>{summary.detail}</small>
            {summary.outputs.length > 0 && <button type="button" onClick={() => summary.outputs[0] ? void invoke("open_output_parent", { path: summary.outputs[0] }) : void invoke("open_output_folder")}><FolderOpen size={14} /> Show output location</button>}
            <button type="button" onClick={() => { handledDropRef.current = false; setSummary(null); setNotice(null); }}>Back to wheel</button>
          </div>
        )}
      </div>
    </main>
  );
}
