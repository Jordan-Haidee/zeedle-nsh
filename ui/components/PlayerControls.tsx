import { Image, View, signal, token } from "nsh";
import type { Signal } from "nsh";
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
  const setVolumeAt = (y: number) => {
    props.onVolumeChange(Math.max(0, Math.min(1, 1 - y / 55)));
  };
  return (
    <View
      class="relative h-17 w-full items-center justify-center"
      onMouseEnter={() => hovered.set(true)}
      onMouseLeave={() => hovered.set(false)}
    >
      {hovered() ? (
        <View
          accessibilityLabel={props.volumeLabel}
          accessibilityValue={{ min: 0, max: 1, now: props.volume() }}
          role="adjustable"
          onPress={(event) => setVolumeAt(event?.position.y ?? 27.5)}
          class="absolute right-3.25 top-1.5 h-13.75 w-5"
        >
          <View
            class="pointer-events-none absolute left-2 top-0 h-13.75 w-1 rounded-full"
            style={{ backgroundColor: token("track") }}
          >
            <View
              class="absolute bottom-0 left-0 w-1 rounded-full"
              style={{ height: 55 * props.volume(), backgroundColor: token("primary") }}
            />
            <View
              class="absolute -left-0.75 h-2.5 w-2.5 rounded-full border"
              style={{
                bottom: Math.max(0, Math.min(50, 55 * props.volume() - 5)),
                borderColor: token("page"),
                backgroundColor: token("primary"),
              }}
            />
          </View>
        </View>
      ) : null}
      <View
        accessibilityLabel={props.muteLabel}
        onPress={() => props.onVolumeChange(props.volume() > 0 ? 0 : 0.8)}
        class="flex h-7.5 w-7 items-center justify-center hover:bg-hover active:bg-pressed"
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
