import { useComputedColorScheme } from "@mantine/core";
import ProtovLogo from "../../assets/protov_logo.svg";

type LogoProps = React.ImgHTMLAttributes<HTMLImageElement> & {
  width?: number | string;
};

export function Logo({ ...props }: LogoProps) {
  const scheme = useComputedColorScheme();

  return (
    <img
      {...props}
      src={ProtovLogo}
      style={{
        filter: scheme === "dark" ? "invert(0)" : "invert(1)",
        opacity: 0.9,
        transition: "filter 150ms ease",
        ...(props.style ?? {}),
      }}
    />
  );
}