import { useApi, useApiDelete } from "@/client";
import { useUser } from "@/services/auth";
import { ActionIcon, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconTrash } from "@tabler/icons-react";
import { useRouter } from "@tanstack/react-router";

export const DeleteEvent = ({ eventId }: { eventId: string }) => {
  const user = useUser();
  const { data: event, isLoading } = useApi(`/api/events/{eid}`, {
    path: { eid: eventId },
    enabled: !!eventId,
  });
  const { navigate } = useRouter();
  const { mutate, isPending } = useApiDelete("/api/events/{eid}", { path: { eid: eventId } }, async () => {
    close();
    await navigate({ to: "/" });
  });
  const [opened, { toggle, close }] = useDisclosure(false);

  if (!user.roles.includes("ec")) return null;

  return (
    <>
      <ActionIcon variant="subtle" color="red" aria-label="Settings" onClick={toggle}>
        <IconTrash size={18} />
      </ActionIcon>
      <Modal opened={opened} onClose={close} title="Delete Event">
        <Stack>
          <Text>Do you want to delete event {event?.title}?</Text>
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
