import { FloatingPosition, Text, Tooltip } from "@mantine/core";
import { format, formatDistanceToNow, isFuture, isPast } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

export const DateTime = ({
  children,
  noDistance,
  noDate,
  position,
}: {
  children?: string | Date;
  noDistance?: boolean;
  noDate?: boolean;
  position?: FloatingPosition;
}) => {
  if (!children) return null;

  const time = typeof children === "string" ? new Date(children) : children;
  return (
    <>
      <Tooltip label={format(time, "yyyy-MM-dd HH:mm zzzz")} position={position}>
        <Text component="span">
          {formatInTimeZone(time, "UTC", noDate ? "HHmm" : "yyyy-MM-dd HH:mm")}
          <Text component="span" c="gray" fz="sm">
            Z
          </Text>
        </Text>
      </Tooltip>
      {!noDistance && (
        <Text component="span" ml={4}>
          ({isFuture(time) && "in "}
          {formatDistanceToNow(time)}
          {isPast(time) && " ago"})
        </Text>
      )}
    </>
  );
};
