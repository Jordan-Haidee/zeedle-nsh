import { Slider } from "nsh/base";
import type { Signal } from "nsh";

type ProgressSliderProps = {
  position: Signal<number>;
  duration: Signal<number>;
  label: string;
  onChange: (position: number) => void;
  onChangeEnd: (position: number) => void;
};

export function ProgressSlider(props: ProgressSliderProps) {
  return (
    <Slider
      accessibilityLabel={props.label}
      value={props.duration() > 0 ? Math.max(0, Math.min(props.duration(), props.position())) : 0}
      min={0}
      max={props.duration()}
      step={0}
      onChange={({ value }) => props.onChange(value as number)}
      onChangeEnd={({ value }) => props.onChangeEnd(value as number)}
      class="my-[-10px] h-6 w-full"
      trackClass="h-6 justify-center bg-transparent"
      indicatorClass="h-1 bg-track"
      rangeClass="h-[3px] bg-primary"
      thumbClass="size-5 rounded-full border-[3px] border-track-thumb-border bg-primary"
    />
  );
}
