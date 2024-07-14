import { Text, Tooltip } from "@mantine/core";
import { format, formatDistanceToNow, isFuture, isPast } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

export const DateTime = ({
  children,
  noDistance,
  noDate,
}: {
  children?: string | Date;
  noDistance?: boolean;
  noDate?: boolean;
}) => {
  if (!children) return null;

  const time = typeof children === "string" ? new Date(children) : children;
  return (
    <>
      <Tooltip label={format(time, noDate ? "HH:mm zzzz" : "yyyy-MM-dd HH:mm zzzz")}>
        <Text component="span">{formatInTimeZone(time, "UTC", noDate ? "HH:mm" : "yyyy-MM-dd HH:mm")}Z</Text>
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
