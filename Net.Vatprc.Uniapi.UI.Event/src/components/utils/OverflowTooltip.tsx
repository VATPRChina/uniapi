/**
 * @see https://github.com/chakra-ui/chakra-ui/issues/4202#issuecomment-1352663485
 */
import { Heading, HeadingProps, Tooltip, TooltipProps } from "@chakra-ui/react";
import React, { useEffect, useRef, useState } from "react";

interface OverflowTooltipProps extends HeadingProps {
  children: React.ReactNode;
  tooltipProps: Omit<TooltipProps, "children">;
}

export const OverflowTooltip = (props: OverflowTooltipProps) => {
  const { children, tooltipProps, ...headingProps } = props;
  const textElementRef = useRef<HTMLDivElement>(null);
  const [isOverflown, setIsOverflown] = useState(false);

  /**
   * Check whether the element is overflow or not
   */
  const compareSize = () => {
    const element = textElementRef.current;

    const compare = element
      ? element.offsetWidth < element.scrollWidth || element.offsetHeight < element.scrollHeight
      : false;

    setIsOverflown(compare);
  };

  useEffect(() => {
    compareSize();
  }, []);

  return (
    <Tooltip label={children} isDisabled={!isOverflown} {...tooltipProps}>
      <Heading ref={textElementRef} {...headingProps}>
        {children}
      </Heading>
    </Tooltip>
  );
};
