import { NavbarSimple } from './ui/navbar';

import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { DevicesPage } from './ui/pages/devices';
import { ControlsPage } from './ui/pages/controls';
import { MeasurementsPage } from './ui/pages/measurements';
import { GraphsPage } from './ui/pages/graphs';
import { TelemetryPage } from './ui/pages/telemetry';
import { useDisclosure } from '@mantine/hooks';
import { AppShell, Burger, Group } from '@mantine/core';
import { Logo } from './ui/components/logo';
import { ConnectedDevicesChip } from './ui/components/connected_devices_chip';
import { DisableOutput } from './ui/components/disable_output';

export function App() {
    const [mobileOpened, { toggle: toggleMobile }] = useDisclosure();
    const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);

    return (
        <BrowserRouter>
            <AppShell
                padding="md"
                header={{ height: 60 }}
                navbar={{
                    width: 275,
                    breakpoint: "sm",
                    collapsed: { mobile: !mobileOpened, desktop: !desktopOpened }
                }}
            >
                <AppShell.Header>
                    <Group h="100%" px="md" justify='space-between'>
                        <Group h="100%" px="md">
                            <Burger opened={mobileOpened} onClick={toggleMobile} hiddenFrom="sm" size="sm" />
                            <Burger opened={desktopOpened} onClick={toggleDesktop} visibleFrom="sm" size="sm" />
                            <Logo height="50%" style={{ paddingTop: "7px" }} />
                        </Group>
                        <Group>
                            <Group visibleFrom="sm">
                                <ConnectedDevicesChip />
                            </Group>
                            <DisableOutput />
                        </Group>
                    </Group>
                </AppShell.Header>
                <AppShell.Navbar p="sm">
                    <NavbarSimple />
                </AppShell.Navbar>
                <AppShell.Main>
                    <Routes>
                        <Route path="/" element={<Navigate to="/devices" replace />} />
                        <Route path="/devices" element={<DevicesPage />} />
                        <Route path="/controls" element={<ControlsPage />} />
                        <Route path="/measurements" element={<MeasurementsPage />} />
                        <Route path="/graphs" element={<GraphsPage />} />
                        <Route path="/telemetry" element={<TelemetryPage />} />
                    </Routes>
                </AppShell.Main>
            </AppShell>
        </BrowserRouter>
    );
}
