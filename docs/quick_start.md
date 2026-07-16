# Quick start

ProtoV MINI can be used as a standalone power supply or connected to a
computer for control through [ProtoV App](https://protov.app).

| Setup | What you need | Computer control |
| --- | --- | --- |
| [USB-C PD supply only](#option-1-usb-c-pd-supply-only) | USB-C PD supply and CC-capable USB-C cable | No |
| [USB-C PD supply and computer](#option-2-usb-c-pd-supply-and-computer) | USB-C PD supply, computer, USB-C cables, and USB-C OTG splitter | Yes |

Before starting, check the lists of [tested USB-C power
sources](compatibility.md#usb-c-power-sources) and
[ProtoV App-compatible platforms](compatibility.md#protov-app).

## Option 1: USB-C PD supply only

Use this setup when you want to control ProtoV MINI from its display and
buttons without connecting it to a computer.

1. Connect a USB-C to USB-C cable to a USB-C PD power supply.
2. Connect the other end of the cable to ProtoV MINI's USB-C connector.
3. Wait for the splash screen and confirm that the boot-time power check shows
   a check mark.
4. Check the status in the upper-left corner of the display.

> [!IMPORTANT]
> Use a USB-C cable with Configuration Channel (CC) conductors. A cable without
> the CC connection cannot perform USB-C Power Delivery negotiation.

The display identifies the detected connection:

| Display status | Meaning |
| --- | --- |
| **USB PD** | A USB Power Delivery contract was negotiated |
| **USB 2.0** | The device is receiving standard USB power without a USB PD contract |
| **Linked** | A USB data connection to a computer is active |

Compare the negotiated voltage, current, and power shown on the display with
the power supply's ratings. See [USB-C power-source
compatibility](compatibility.md#usb-c-power-sources) for tested examples and
negotiated contracts.

> [!TIP]
> If the display shows **USB 2.0** when you expected **USB PD**, verify that
> both the supply and cable support USB Power Delivery, then reconnect the
> cable. While ProtoV MINI supports both USB-C connector orientations,
> some lower-quality cables may behave differently when flipped, potentially
> affecting connection stability.

## Option 2: USB-C PD supply and computer

Use a USB-C OTG splitter to power ProtoV MINI from a USB-C PD supply while
connecting its USB data interface to a computer.

> [!IMPORTANT]
> Connect both the power supply and the computer to the splitter **before**
> connecting the splitter to ProtoV MINI. Connecting the branches in a
> different order can cause an incorrect power negotiation.

1. With the splitter disconnected from ProtoV MINI, connect the USB-C PD supply
   to the splitter's power-input port.
2. Connect the computer's USB cable to the splitter's non-power, data-capable
   port.
3. Connect the splitter to ProtoV MINI's USB-C connector.
4. Wait for the splash screen and confirm that the boot-time power check shows
   a check mark.
5. Check the upper-left corner of the display for **USB PD** and **Linked**.
6. Verify that the displayed negotiated voltage, current, and power match the
   expected capabilities of the power supply.
7. Open [ProtoV App](https://protov.app) in a
   [supported browser](compatibility.md#protov-app).
8. Select **Lab**, then select **Connect**.
9. In the browser's device picker, select the serial device enumerated by
   ProtoV MINI and confirm the connection.

> [!TIP]
> If ProtoV MINI does not appear in the device picker, confirm that the
> computer is connected to the splitter's data-capable port and that the
> display shows **Linked**. Browser and operating-system support is documented
> in [ProtoV App compatibility](compatibility.md#protov-app).

For device dimensions, controls, and hardware design resources, see the
[hardware guide](hardware.md).
