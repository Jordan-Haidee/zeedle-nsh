export function timeLabel(seconds: number) {
  const value = Math.max(0, Math.floor(seconds || 0));
  return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
}

export function sameSongPath(left: string | null | undefined, right: string | null | undefined) {
  return left?.replaceAll("\\", "/") === right?.replaceAll("\\", "/");
}
