import { Image, type ImageAsset } from "nsh";
import arrowDown from "./icons/arrow-down.svg";
import arrowUp from "./icons/arrow-up.svg";
import captions from "./icons/captions.svg";
import check from "./icons/check.svg";
import circleAlert from "./icons/circle-alert.svg";
import circleHelp from "./icons/circle-help.svg";
import listMusic from "./icons/list-music.svg";
import moon from "./icons/moon.svg";
import minus from "./icons/minus.svg";
import panelLeftClose from "./icons/panel-left-close.svg";
import panelLeftOpen from "./icons/panel-left-open.svg";
import pause from "./icons/pause.svg";
import play from "./icons/play.svg";
import repeat from "./icons/repeat.svg";
import repeat2 from "./icons/repeat-2.svg";
import search from "./icons/search.svg";
import settings from "./icons/settings.svg";
import shuffle from "./icons/shuffle.svg";
import skipBack from "./icons/skip-back.svg";
import skipForward from "./icons/skip-forward.svg";
import square from "./icons/square.svg";
import sun from "./icons/sun.svg";
import volume2 from "./icons/volume-2.svg";
import x from "./icons/x.svg";

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
