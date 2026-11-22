import type { TrayOptions, WindowRef } from "nsh/desktop";

export function createTrayOptions(
  window: Pick<WindowRef, "show" | "focus">,
  rescanSongs: () => void,
  translate: (key: string) => string,
): TrayOptions {
  const showWindow = () => {
    window.show();
    window.focus();
  };

  return {
    tooltip: "Zeedle",
    onClick: showWindow,
    menu: [
      { label: translate("Show window"), onSelect: showWindow },
      {
        label: translate("Rescan songs"),
        onSelect: rescanSongs,
      },
      { type: "separator" },
      { label: translate("Exit"), role: "quit" },
    ],
  };
}
