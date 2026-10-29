import { Image, type ImageAsset } from "nsh";
import arrowDown from "./assets/icons/arrow-down.svg";
import arrowUp from "./assets/icons/arrow-up.svg";
import captions from "./assets/icons/captions.svg";
import check from "./assets/icons/check.svg";
import circleAlert from "./assets/icons/circle-alert.svg";
import circleHelp from "./assets/icons/circle-help.svg";
import listMusic from "./assets/icons/list-music.svg";
import moon from "./assets/icons/moon.svg";
import minus from "./assets/icons/minus.svg";
import panelLeftClose from "./assets/icons/panel-left-close.svg";
import panelLeftOpen from "./assets/icons/panel-left-open.svg";
import pause from "./assets/icons/pause.svg";
import play from "./assets/icons/play.svg";
import repeat from "./assets/icons/repeat.svg";
import repeat2 from "./assets/icons/repeat-2.svg";
import search from "./assets/icons/search.svg";
import settings from "./assets/icons/settings.svg";
import shuffle from "./assets/icons/shuffle.svg";
import skipBack from "./assets/icons/skip-back.svg";
import skipForward from "./assets/icons/skip-forward.svg";
import square from "./assets/icons/square.svg";
import sun from "./assets/icons/sun.svg";
import volume2 from "./assets/icons/volume-2.svg";
import x from "./assets/icons/x.svg";

const icons = {
  arrowDown,
  arrowUp,
  captions,
  check,
  circleAlert,
  circleHelp,
  listMusic,
  moon,
  minus,
  panelLeftClose,
  panelLeftOpen,
  pause,
  play,
  repeat,
  repeat2,
  search,
  settings,
  shuffle,
  skipBack,
  skipForward,
  square,
  sun,
  volume2,
  x,
} satisfies Record<string, ImageAsset>;

export type LucideName = keyof typeof icons;

export function LucideIcon(props: { name: LucideName; size?: number; color: string }) {
  return <Image src={icons[props.name]} style={{ width: props.size ?? 18, height: props.size ?? 18, tintColor: props.color }} />;
}
