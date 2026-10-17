// Reclaim: per-FileCategory label, colour and icon. `color` resolves the theme CSS variable at access
// time to a concrete colour string, so it works in inline styles AND in ECharts (canvas can't read var()).
import { Archive, Database, Disc3, File, FileCode, FileText, Image, Music, ScrollText, Terminal, Video, type LucideIcon } from "lucide-react";
import type { FileCategory } from "../lib/types";

function resolve(cat: FileCategory): string {
  if (typeof document === "undefined") return "#9ca3af";
  return getComputedStyle(document.documentElement).getPropertyValue(`--cat-${cat}`).trim() || "#9ca3af";
}

function meta(cat: FileCategory, label: string, icon: LucideIcon) {
  return { label, icon, get color() { return resolve(cat); } };
}

export const categoryMeta: Record<FileCategory, { label: string; color: string; icon: LucideIcon }> = {
  video: meta("video", "Video", Video),
  audio: meta("audio", "Audio", Music),
  image: meta("image", "Images", Image),
  document: meta("document", "Documents", FileText),
  archive: meta("archive", "Archives", Archive),
  diskImage: meta("diskImage", "Disk images", Disc3),
  code: meta("code", "Code", FileCode),
  executable: meta("executable", "Executables", Terminal),
  database: meta("database", "Databases", Database),
  log: meta("log", "Logs", ScrollText),
  other: meta("other", "Other", File),
};
