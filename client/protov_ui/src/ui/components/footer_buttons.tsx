import { ActionIcon, Tooltip } from "@mantine/core";
import { IconBrandGithub } from "@tabler/icons-react";

import cx from 'clsx';
import classes from './theme_toggle.module.css';

export function DocumentationButton() {
    return <Tooltip label="Documentation on Github">
        <ActionIcon
            variant="default"
            size="lg"
            radius="md"
            aria-label="Documentation"
            component="a"
            href="https://github.com/thataquarel/protovolt"
            target="_blank"
            rel="noreferrer"
        >
            <IconBrandGithub className={cx(classes.icon)} />
        </ActionIcon>
    </Tooltip>
}