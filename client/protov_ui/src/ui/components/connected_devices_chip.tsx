import { Badge, Group } from "@mantine/core";
import { IconTransfer, IconUsb } from "@tabler/icons-react";

export function ConnectedDevicesChip () {
    return <Group>
        <Badge variant="outline" color="grey" leftSection={<IconTransfer size={14}/>}>Stream: active</Badge>
        <Badge variant="outline" color="grey" leftSection={<IconUsb size={14}/>}>Connected: 1</Badge>
    </Group>
}
