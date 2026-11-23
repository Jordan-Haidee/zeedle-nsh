export function closeWindow(keepInTray: boolean, hide: () => void, quit: () => void) {
  if (keepInTray) hide();
  else quit();
}
