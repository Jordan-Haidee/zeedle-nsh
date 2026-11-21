import { Image, View, signal, token } from "nsh";
import type { Signal } from "nsh";
import { Slider } from "nsh/base";
import { Music } from "zeedle-nsh/native";
import { LucideIcon, type LucideName } from "../icons";

type TransportButtonProps = {
  testID: string;
  label: string;
  name: LucideName;
  color: string;
  onPress: () => void;
  radius?: number;
};

function TransportIcon(props: { name: LucideName; color: string; visible?: boolean }) {
  return (
    <View
      class="pointer-events-none absolute left-1.25 top-1.25 size-6.5 items-center justify-center"
      style={{ opacity: props.visible === false ? 0 : 1 }}
    >
      <Image src={Music.transportIcon(props.name, props.color)} class="size-6.5 object-contain" />
    </View>
  );
}

export function PlayerTransportButton(props: TransportButtonProps) {
  return (
    <View
      testID={props.testID}
      accessibilityLabel={props.label}
      onPress={props.onPress}
      class="relative items-center justify-center hover:bg-hover active:bg-pressed transition-colors duration-120 ease-out motion-reduce:transition-none"
      style={{ width: 36, height: 36, borderRadius: props.radius ?? 9 }}
    >
      {props.name === "play" || props.name === "pause" ? (
        <>
          <TransportIcon name="play" color={props.color} visible={props.name === "play"} />
          <TransportIcon name="pause" color={props.color} visible={props.name === "pause"} />
        </>
      ) : (
        <TransportIcon name={props.name} color={props.color} />
      )}
    </View>
  );
}

type VolumeControlProps = {
  volume: Signal<number>;
  volumeLabel: string;
  muteLabel: string;
  onVolumeChange: (value: number) => void;
};

export function VolumeControl(props: VolumeControlProps) {
  const hovered = signal(false);
  const dragging = signal(false);
  const previewVolume = signal(props.volume());
  return (
    <View
      class="relative h-17 w-full items-center justify-center"
      onMouseEnter={() => {
        previewVolume.set(props.volume());
        hovered.set(true);
      }}
      onMouseLeave={() => {
        if (!dragging()) hovered.set(false);
      }}
    >
      {hovered() ? (
        <Slider
          accessibilityLabel={props.volumeLabel}
          value={previewVolume()}
          min={0}
          max={1}
          step={0}
          axis="vertical"
          onChange={({ value }) => {
            if (typeof value === "number") {
              dragging.set(true);
              previewVolume.set(value);
            }
          }}
          onChangeEnd={({ value }) => {
            if (typeof value === "number") {
              previewVolume.set(value);
              props.onVolumeChange(value);
            }
            dragging.set(false);
          }}
          class="absolute right-3.25 top-1.5 h-13.75 w-5"
          trackClass="h-13.75 w-5 justify-center bg-transparent"
          indicatorClass="h-full w-1 rounded-full bg-track"
          rangeClass="w-1 rounded-full bg-primary"
          thumbClass="size-2.5 rounded-full border border-page bg-primary"
        />
      ) : null}
      <View
        accessibilityLabel={props.muteLabel}
        onPress={() => props.onVolumeChange(props.volume() > 0 ? 0 : 0.8)}
        class="flex h-7.5 w-7 items-center justify-center active:bg-pressed"
      >
        <LucideIcon
          name="volume2"
          size={20}
          color={hovered() ? token("primary") : token("text-secondary")}
        />
      </View>
    </View>
  );
}
