import { Text, Card, Container, SimpleGrid, Skeleton, Group, Stack, Paper, Chip, Code, Badge, NumberInput, ActionIcon, Tooltip } from "@mantine/core";

import classes from "./controls.module.css";
import { Channel } from "../components/channel_chip";
import { useState } from "react";
import { IconCheck, IconChecks, IconNumber, IconUsb } from "@tabler/icons-react";


export function ControlsPage() {
    return (
        <Container>
            <SimpleGrid
                type="container"
                cols={{ base: 1, '300px': 1, '700px': 2, '900px': 3 }}
                spacing={{ base: "md" }}
            >
                <Skeleton radius="md" animate visible={false} >
                    <ControlsCard
                        name="ProtoV MINI"
                        sn="SN 550e8400"
                        port="/dev/ttyUSB0"
                        channels={[
                            { identifier: "A", color: "red", voltage: 3.3, current: 0.50, active: true },
                            { identifier: "B", color: "blue", voltage: 1.8, current: 0.10, active: true },
                        ]} />
                </Skeleton>
                <Skeleton radius="md" animate visible={false}>
                    <ControlsCard
                        name="ProtoV MINI"
                        port="COM0"
                        sn="SN 32983fe4"
                        channels={[
                            { identifier: "A", color: "yellow", voltage: 20.00, current: 5.00, active: true },
                            { identifier: "B", color: "green", voltage: 5.00, current: 1.00, active: true },
                        ]} />
                </Skeleton>
                <Skeleton radius="md" animate />
                <Skeleton radius="md" animate />
            </SimpleGrid>
        </Container>
    )
}

interface ControlsCardProps {
    name: string,
    port: string,
    sn: string,
    channels: Channel[],
}

interface NumberSettingProps {
    placeholder: string,
    unit: string,
    min: number,
    max: number,
    decimals: number,
}

export function NumberSetting({
    placeholder,
    unit,
    min,
    max,
    decimals
}: NumberSettingProps) {
    return <NumberInput
        placeholder={placeholder}
        leftSection={unit}
        decimalScale={decimals}
        fixedDecimalScale
        min={min}
        max={max}
        mt="xs"
    />
}

export function ChannelControls({
    identifier,
    color,
    voltage,
    current,
    active }: Channel
) {
    const [checked, setChecked] = useState(active);

    return <Paper shadow="xs" pt="md" pl="md" pr="md" pb="md" withBorder>
        <Chip
            checked={checked}
            onChange={(value) => setChecked(value)}
            color={color}
            variant="light"
        >
            {`CH${identifier}`}
        </Chip>

        <Stack>
            <Text className={classes.readings}>
                {voltage}
            </Text>

            <Text className={classes.readings}>
                {current}
            </Text>
        </Stack>
        <Tooltip label="Target voltage and current on output">
            <Text mt="md" className={classes.label} c="dimmed">
                SETPOINT
            </Text>
        </Tooltip>
        <NumberSetting placeholder="0.000 - 20.000" unit="V" min={0} max={20} decimals={3} />
        <NumberSetting placeholder="0.000 - 5.000" unit="A" min={0} max={5} decimals={3} />

        <Tooltip label="Absolute maximum voltage and current before safety shutdown">
            <Text mt="md" className={classes.label} c="dimmed">
                PROTECTION
            </Text>
        </Tooltip>
        <NumberSetting placeholder="0.000 - 20.000" unit="V" min={0} max={20} decimals={3} />
        <NumberSetting placeholder="0.000 - 5.000" unit="A" min={0} max={5} decimals={3} />
    </Paper>
}

export function ControlsCard({
    name,
    port,
    sn,
    channels
}: ControlsCardProps) {
    const channel_details = channels.map((ch) => (
        <ChannelControls key={ch.identifier} {...ch} />
    ));

    return (
        <Card
            shadow="sm"
            padding="lg"
            radius="md"
            withBorder
            style={{ height: "100%", display: "flex", flexDirection: "column" }}
            mb="lg"
        >
            <Group justify="space-between" gap="xs">
                <Text fw={500}>{name}</Text>
                <Code>{port}</Code>
            </Group>

            <Stack justify="space-between" align="stretch">
                <Text size="sm" c="dimmed">
                    Dual-channel DC output adjustment
                </Text>

                <Card.Section className={classes.section}>
                    <Stack>
                        {channel_details}
                    </Stack>
                </Card.Section>
            </Stack>
        </Card>
    );
}