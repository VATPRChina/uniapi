import { invalidatePath, useApi, useApiDelete } from "@/client";
import { useUser } from "@/services/auth";
import { ActionIcon, Button, Group, Popover, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconTrash } from "@tabler/icons-react";
import { useRouter } from "@tanstack/react-router";

export const DeleteSlot = ({ eventId, slotId }: { eventId: string; slotId: string }) => {
  const user = useUser();
  const { navigate } = useRouter();
  const { mutate, isPending } = useApiDelete(
    "/api/events/{eid}/slots/{sid}",
    { path: { eid: eventId, sid: slotId } },
    async () => {
      await invalidatePath("/api/events/{eid}/slots", { eid: eventId });
      close();
      await navigate({ to: "/events/" + eventId });
    },
  );
  const [opened, { toggle, close }] = useDisclosure(false);
  const { data: slot, isLoading } = useApi("/api/events/{eid}/slots/{sid}", {
    path: { eid: eventId, sid: slotId },
    enabled: !!eventId && opened,
  });

  if (!user?.roles.includes("ec")) return null;

  return (
    <Popover opened={opened} onClose={close}>
      <Popover.Target>
        <ActionIcon color="red" onClick={toggle} variant="subtle">
          <IconTrash size={18} />
        </ActionIcon>
      </Popover.Target>
      <Popover.Dropdown>
        <Stack>
          <Text>
            Do you want to delete slot entering {slot?.airspace?.name} at {slot?.enter_at}?
          </Text>
          <Group>
            <Button variant="outline" onClick={toggle}>
              Cancel
            </Button>
            <Button color="red" onClick={() => mutate({})} loading={isPending} disabled={isLoading}>
              Yes
            </Button>
          </Group>
        </Stack>
      </Popover.Dropdown>
    </Popover>
  );
};
