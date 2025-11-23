import {
    Text, Container, SimpleGrid, Skeleton, Stack, NumberInput,
    Paper, Chip, Tooltip,
    Card,
    Group,
    useMantineTheme,
    MantineColor,
    NumberFormatter,
    Switch,
    Badge
} from "@mantine/core";

import classes from "./controls.module.css";
import { Channel } from "../components/channel_chip";
import { useState } from "react";
import { IconBolt, IconMoonStars, IconSun } from "@tabler/icons-react";

export function ControlsPage() {
    return (
        <Container>
            <SimpleGrid
                type="container"
                cols={{ base: 1, '300px': 1, '850px': 2 }}
                spacing={{ base: "md" }}
            >
                <Skeleton radius="md" visible={false} animate={false}>
                    <ChannelCard
                        identifier="A"
                        color="red"
                        voltage={3.3}
                        current={0.50}
                        active={true}
                    />
                </Skeleton>

                <Skeleton radius="md" visible={false} animate={false}>
                    <ChannelCard
                        identifier="B"
                        color="blue"
                        voltage={1.8}
                        current={0.10}
                        active={true}
                    />
                </Skeleton>

                <Skeleton radius="md" visible={false} animate={false}>
                    <ChannelCard
                        identifier="C"
                        color="yellow"
                        voltage={1.8}
                        current={0.10}
                        active={true}
                    />
                </Skeleton>

                <Skeleton radius="md" visible={false} animate={false}>
                    <ChannelCard
                        identifier="D"
                        color="green"
                        voltage={1.8}
                        current={0.10}
                        active={true}
                    />
                </Skeleton>

                <Skeleton radius="md" animate />
                <Skeleton radius="md" animate />
            </SimpleGrid>
        </Container>
    );
}

interface NumberSettingProps {
    placeholder: string;
    unit: string;
    min: number;
    max: number;
    decimals: number;
}

export function NumberSetting({
    placeholder,
    unit,
    min,
    max,
    decimals,
}: NumberSettingProps) {
    return (
        <NumberInput
            placeholder={placeholder}
            leftSection={unit}
            decimalScale={decimals}
            fixedDecimalScale
            min={min}
            max={max}
            mt="xs"
        />
    );
}


interface ReadingProps {
    title: string,
    value: number,
    decimals: number,
    unit: string,
}

export function Reading({ title, value, decimals, unit }: ReadingProps) {
    const offset = 10 ** -(decimals + 1);

    return <Stack>
        <Text>{title}</Text>
        <Paper>
            <Group justify="space-between">
                <Text className={classes.readings}>
                    <NumberFormatter value={value + offset} decimalScale={decimals} />
                </Text>
                <Text className={classes.unit} mr="xs">{unit}</Text>
            </Group>
        </Paper>
    </Stack>
}

export function ChannelCard({
    identifier,
    color,
    voltage,
    current,
    active,
}: Channel) {
    const theme = useMantineTheme();
    let border_color = theme.colors[color as MantineColor][5];

    return (
        <Card
            shadow="sm"
            padding="lg"
            radius="md"
            withBorder
            style={{
                height: "100%",
                display: "flex",
                flexDirection: "column",
                borderColor: border_color
            }}
        >
            <Group justify="space-between" mb="md">
                <Badge color={color} size="md">
                    {`Channel ${identifier}`}
                </Badge>

                <Switch
                    size="md"
                    color={color}
                    onLabel={<IconBolt size={16} stroke={2.5} />}
                />
            </Group>

            <Group grow>
                <Reading title="Voltage" value={voltage} decimals={3} unit={"V"} />
                <Reading title="Current" value={current} decimals={3} unit={"A"} />
                <Reading title="Power" value={voltage * current} decimals={3} unit={"W"} />
            </Group>

            <Tooltip label="Target voltage and current on output">
                <Text mt="md" className={classes.label} c="dimmed">
                    SETPOINT
                </Text>
            </Tooltip>

            <Group grow>
                <NumberSetting placeholder="0.000 - 20.000" unit="V" min={0} max={20} decimals={3} />
                <NumberSetting placeholder="0.000 - 5.000" unit="A" min={0} max={5} decimals={3} />
            </Group>

            <Tooltip label="Absolute maximum voltage and current before safety shutdown">
                <Text mt="md" className={classes.label} c="dimmed">
                    PROTECTION
                </Text>
            </Tooltip>

            <Group grow>
                <NumberSetting placeholder="0.000 - 20.000" unit="V" min={0} max={20} decimals={3} />
                <NumberSetting placeholder="0.000 - 5.000" unit="A" min={0} max={5} decimals={3} />
            </Group>
        </Card>
    );
}
0.000 - 20.000
