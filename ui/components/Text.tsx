import { split, Text as NshText } from "nsh";
import { AppConfig } from "zeedle-nsh/native";

type NshTextProps = Parameters<typeof NshText>[0];
type TextProps = NshTextProps & { numberOfLines?: number; children?: NshTextProps["children"] };

export function Text(props: TextProps) {
  const [own, rest] = split(props, ["numberOfLines", "children"] as const);
  const truncate = (own.numberOfLines ?? 1) === 1;
  const lineStyle =
    own.numberOfLines != null && own.numberOfLines !== 1
      ? { numberOfLines: own.numberOfLines }
      : undefined;
  return (
    <NshText
      {...rest}
      class={[props.class, truncate ? "truncate" : ""].filter(Boolean).join(" ")}
      style={[{ fontFamily: AppConfig.defaultFontFamily() }, props.style, lineStyle]}
    >
      {own.children}
    </NshText>
  );
}
