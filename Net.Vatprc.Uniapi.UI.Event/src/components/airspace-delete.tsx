import { invalidatePath, useApi, useApiDelete } from "@/client";
import { useUser } from "@/services/auth";
import { ActionIcon, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconTrash } from "@tabler/icons-react";
import { useRouter } from "@tanstack/react-router";

export const DeleteAirspace = ({ eventId, airspaceId }: { eventId: string; airspaceId: string }) => {
  const [opened, { toggle, close }] = useDisclosure(false);

  const user = useUser();
  const { data: airspace, isLoading } = useApi(`/api/events/{eid}/airspaces/{aid}`, {
    path: { eid: eventId, aid: airspaceId },
    enabled: !!eventId && opened,
  });
  const { navigate } = useRouter();
  const { mutate, isPending } = useApiDelete(
    "/api/events/{eid}/airspaces/{aid}",
    { path: { eid: eventId, aid: airspaceId } },
    async () => {
      await invalidatePath("/api/events/{eid}/airspaces", { eid: eventId });
      close();
      await navigate({ to: "/events/" + eventId });
    },
  );

  if (!user.roles.includes("event_coordinator")) return null;

  return (
    <>
      <ActionIcon variant="subtle" color="red" aria-label="Settings" onClick={toggle}>
        <IconTrash size={18} />
      </ActionIcon>
      <Modal opened={opened} onClose={close} title="Delete Event">
        <Stack>
          <Text>Do you want to delete airspace {airspace?.name}?</Text>
          <Group>
            <Button onClick={toggle}>Cancel</Button>
            <Button color="red" onClick={() => mutate({})} loading={isPending} disabled={isLoading}>
              Yes
            </Button>
          </Group>
        </Stack>
      </Modal>
    </>
  );
};
