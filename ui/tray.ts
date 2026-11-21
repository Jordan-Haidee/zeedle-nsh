import type { TrayOptions, WindowRef } from "nsh/desktop";

export function createTrayOptions(
  window: Pick<WindowRef, "show" | "focus">,
  rescanSongs: () => void,
  translate: (english: string, chinese: string) => string,
): TrayOptions {
  const showWindow = () => {
    window.show();
    window.focus();
  };

  return {
    tooltip: "Zeedle",
    onClick: showWindow,
    menu: [
      { label: translate("Show window", "显示窗口"), onSelect: showWindow },
      {
        label: translate("Rescan songs", "重新扫描歌曲"),
        onSelect: rescanSongs,
      },
      { type: "separator" },
      { label: translate("Exit", "退出"), role: "quit" },
    ],
  };
}
