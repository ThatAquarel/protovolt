import { useState } from 'react';
import { ActionIcon, Tooltip } from '@mantine/core';
import { IconPower } from '@tabler/icons-react';
import cx from 'clsx';
import classes from './theme_toggle.module.css';

export function DisableOutput() {
  const [on, setOn] = useState(true);

  const handleClick = () => setOn((prev) => !prev);

  return (
    <Tooltip label="Disable All">
      <ActionIcon
        onClick={handleClick}
        variant={on ? "filled" : "outline"}
        size="lg"
        radius="md"
        aria-label="Disable All"
        color={on ? "red" : "grey"}
      >
        <IconPower
          className={cx(classes.icon)}
          stroke={1.5}
        />
      </ActionIcon>
    </Tooltip>
  );
}
