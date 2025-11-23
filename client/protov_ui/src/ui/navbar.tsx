import { useState } from 'react';
import {
    IconAdjustments,
    IconBolt,
    IconDeviceDesktopAnalytics,
    IconGraph,
    IconHeartbeat,
} from '@tabler/icons-react';
import { Code, Group } from '@mantine/core';

import classes from './navbar.module.css';
import { ActionToggle } from './components/theme_toggle';
import { Logo } from './components/logo';
import { NavLink } from 'react-router-dom';
import { DocumentationButton } from './components/footer_buttons';

const data = [
    { link: '/devices', label: 'Devices', icon: IconBolt },
    { link: '/controls', label: 'Controls', icon: IconAdjustments },
    { link: '/measurements', label: 'Measurements', icon: IconDeviceDesktopAnalytics },
    { link: '/graphs', label: 'Graphs', icon: IconGraph },
    { link: '/telemetry', label: 'Telemetry', icon: IconHeartbeat },
];

export function NavbarSimple() {
    const [active, setActive] = useState('Devices');

    const links = data.map((item) => (
        <NavLink
            key={item.label}
            to={item.link}
            className={classes.link}
            data-active={item.label === active || undefined}
            onClick={(_) => {
                setActive(item.label);
            }}
        >
            <item.icon className={classes.linkIcon} stroke={1.5} />
            <span>{item.label}</span>
        </NavLink>
    ));

    return (
        <nav className={classes.navbar}>
            <div className={classes.navbarMain}>
                <Group className={classes.header} justify="space-between">
                    <Logo width="30%" />
                </Group>
                {links}
            </div>

            <div className={classes.footer}>
                <Group justify="space-between">

                    <Group gap="xs">
                        <ActionToggle />
                        <DocumentationButton />
                    </Group>

                    <Code fw={700}>v0.1.0</Code>
                </Group>
            </div>
        </nav>
    );
}
