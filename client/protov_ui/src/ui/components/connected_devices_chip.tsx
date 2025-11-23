import { Badge, Group } from "@mantine/core";
import { IconTransfer, IconUsb } from "@tabler/icons-react";

export function ConnectedDevicesChip () {
    return <Group>
        <Badge color="cyan" leftSection={<IconTransfer size={14}/>}>Stream: active</Badge>
        <Badge color="lime" leftSection={<IconUsb size={14}/>}>Connected: 1</Badge>
    </Group>
}
