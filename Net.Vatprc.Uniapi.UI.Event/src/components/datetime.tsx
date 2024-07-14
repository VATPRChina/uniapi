import { Tooltip } from "@mantine/core";
import { format, formatDistanceToNow, isFuture, isPast } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

export const DateTime = ({ children }: { children: string | Date }) => {
  const time = typeof children === "string" ? new Date(children) : children;
  return (
    <>
      <Tooltip label={format(time, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
        <span> {formatInTimeZone(time, "UTC", "yyyy-MM-dd HH:mm")}Z </span>
      </Tooltip>
      ({isFuture(time) && "in "}
      {formatDistanceToNow(time)}
      {isPast(time) && " ago"})
    </>
  );
};
