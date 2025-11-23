import { Chip, NumberFormatter } from "@mantine/core";
import { useState } from "react";

export interface Channel {
    identifier: string;
    color: string;
    voltage: number;
    current: number;
    active: boolean;
}

interface ChannelChipProps {
    channel: Channel
}

// export function ChannelChip({ channel }: ChannelChipProps) {

//     return <Chip defaultChecked={channel.active} color={channel.color} variant="light">
//         {`CH${channel.identifier} - `}
//         <NumberFormatter value={channel.voltage} decimalScale={3} />
//         {"V @ "}
//         <NumberFormatter value={channel.current} decimalScale={3} />
//         {"A"}
//     </Chip>
// }

export function ChannelChip({ channel }: ChannelChipProps) {
  const [checked, setChecked] = useState(channel.active);

  return (
    <Chip
      checked={checked}
      onChange={(value) => setChecked(value)}
      color={channel.color}
      variant="outline"
    >
      {`CH${channel.identifier} - `}
      <NumberFormatter value={channel.voltage} decimalScale={3} />
      {"V @ "}
      <NumberFormatter value={channel.current} decimalScale={3} />
      {"A"}
    </Chip>
  );
}
