import { Badge, Text, Image, Button, Card, Container, Group, SimpleGrid, Skeleton, Stack, Code } from '@mantine/core';

import Image0 from "../../assets/images/protovolt-connected-on-breadboard.jpg";
import Image1 from "../../assets/images/protovolt-connected-to-laptop.jpg";
import Image2 from "../../assets/images/protovolt-powered-by-powerbank.jpg";

const protov_images = [Image0, Image1, Image2];

import classes from "./devices.module.css";
import { IconCode, IconNumber, IconNut } from '@tabler/icons-react';
import { Channel, ChannelChip } from '../components/channel_chip';

export function DevicesPage() {
    return (
        <Container>
            <SimpleGrid
                type="container"
                cols={{ base: 1, '300px': 1, '700px': 2, '900px': 3 }}
                spacing={{ base: "md" }}
            >
                <Skeleton radius="md" animate visible={false} >
                    {/* <Demo></Demo> */}

                    <DeviceCard
                        name="ProtoV MINI"
                        port="/dev/ttyACM0"
                        description="Dual-channel, USB-C powered, credit card-sized lab power supply for electronics prototyping and field testing."
                        imageSrc={protov_images[0]}
                        badges={[
                            { label: "FW v0.1.0", icon: IconCode },
                            { label: "HW rev A.1", icon: IconNut },
                            { label: "SN 550e8400", icon: IconNumber },
                        ]}
                        channels={[
                            { identifier: "A", color: "red", voltage: 3.3, current: 0.50, active: true },
                            { identifier: "B", color: "blue", voltage: 1.8, current: 0.10, active: true },
                        ]}
                        onButtonClick={() => console.log("Go to controls clicked")}
                    />
                </Skeleton>
                <Skeleton radius="md" animate visible={false}>
                    {/* <Demo></Demo> */}
                    <DeviceCard
                        name="ProtoV MINI"
                        port="COM10"
                        description="Dual-channel, USB-C powered, credit card-sized lab power supply for electronics prototyping and field testing."
                        imageSrc={protov_images[1]}
                        badges={[
                            { label: "FW v0.1.1", icon: IconCode },
                            { label: "HW rev B.2", icon: IconNut },
                            { label: "SN 32983fe4", icon: IconNumber },
                        ]}
                        channels={[
                            { identifier: "A", color: "yellow", voltage: 20.00, current: 5.00, active: true },
                            { identifier: "B", color: "green", voltage: 5.00, current: 1.00, active: true },
                        ]}
                        onButtonClick={() => console.log("Go to controls clicked")}
                    />
                </Skeleton>
                <Skeleton radius="md" animate />
                <Skeleton radius="md" animate />
            </SimpleGrid>
        </Container>
    );
}

interface DeviceBadge {
    label: string;
    icon: React.ElementType;
}

interface DeviceCardProps {
    name: string;
    port: string;
    description: string;
    imageSrc: string;
    badges: DeviceBadge[];
    channels: Channel[];
    onButtonClick?: () => void;
}

export function DeviceCard({
    name,
    port,
    description,
    imageSrc,
    badges,
    channels,
    onButtonClick,
}: DeviceCardProps) {
    const details = badges.map((badge) => (
        <Badge
            color="grey"
            variant="transparent"
            key={badge.label}
            leftSection={<badge.icon size={12} />}
        >
            <Code>{badge.label}</Code>
        </Badge>
    ));

    const channel_details = channels.map((ch) => (
        <ChannelChip key={ch.identifier} channel={ch} />
    ));

    return (
        <Card
            shadow="sm"
            padding="lg"
            radius="md"
            withBorder
            style={{ height: "100%", display: "flex", flexDirection: "column" }}
        >
            <Card.Section>
                <Image src={imageSrc} height={150} alt={name} />
            </Card.Section>

            <Group justify="space-between" mt="md" mb="xs">
                <Text fw={500}>{name}</Text>
                <Code>{port}</Code>
            </Group>

            <Stack justify="space-between" align="stretch" style={{ flex: 1 }}>
                <Text size="sm" c="dimmed">
                    {description}
                </Text>

                <Card.Section className={classes.section}>
                    <Text mt="md" className={classes.label} c="dimmed">
                        DETAILS
                    </Text>
                    <Group gap={7} mt={5}>
                        {details}
                    </Group>
                </Card.Section>

                <Card.Section className={classes.section}>
                    <Text mt="md" className={classes.label} c="dimmed">
                        AT A GLANCE
                    </Text>
                    <Group gap={7} mt={5}>
                        {channel_details}
                    </Group>
                </Card.Section>

                <Button
                    variant="outline"
                    color="grey"
                    fullWidth
                    mt="md"
                    radius="md"
                    onClick={onButtonClick}
                >
                    Disable
                </Button>
            </Stack>
        </Card>
    );
}