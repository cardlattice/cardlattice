import type { ButtonHTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "cn";

const variants = cva("button", {
  variants: {
    tone: {
      primary: "button-primary",
      secondary: "button-secondary",
      danger: "button-danger",
      quiet: "button-quiet",
    },
  },
  defaultVariants: { tone: "secondary" },
});
export function Button({
  className,
  tone,
  type = "button",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof variants>) {
  return <button type={type} className={cn(variants({ tone }), className)} {...props} />;
}
