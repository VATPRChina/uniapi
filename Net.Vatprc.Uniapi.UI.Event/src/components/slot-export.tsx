import client from "@/client";
import { useUser } from "@/services/auth";
import { wrapPromiseWithToast } from "@/utils";
import { ActionIcon } from "@mantine/core";
import { IconFileExport } from "@tabler/icons-react";

export const ExportSlot = ({ eventId }: { eventId: string }) => {
  const user = useUser();

  const onClick = async () => {
    const data = await client.GET("/api/events/{eid}/slots/bookings.csv", {
      params: { path: { eid: eventId } },
      parseAs: "blob",
    });
    if (!data?.data) return;
    const url = URL.createObjectURL(data?.data);
    const link = document.createElement("a");
    link.download = `slot_booking.csv`;
    link.href = url;
    link.click();
  };

  if (!user?.roles.includes("ec")) return null;
  return (
    <>
      <ActionIcon variant="subtle" aria-label="Export slots" onClick={wrapPromiseWithToast(onClick)}>
        <IconFileExport size={18} />
      </ActionIcon>
    </>
  );
};
