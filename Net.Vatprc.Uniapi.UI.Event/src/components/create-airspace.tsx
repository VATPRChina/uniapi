import { useApiPost } from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithLog, promiseWithToast } from "@/utils";
import { ActionIcon, Button, MantineSpacing, Modal, Stack, StyleProp, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconPlus } from "@tabler/icons-react";
import { useForm } from "@tanstack/react-form";

export const CreateAirspace = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId: string }) => {
  const user = useUser();
  const { mutateAsync } = useApiPost("/api/events/{eid}/airspaces", { path: { eid: eventId } });
  const form = useForm({
    defaultValues: {
      name: "",
    },
    onSubmit: ({ value }) => {
      promiseWithLog(mutateAsync({ name: value.name }).then(() => close()));
    },
  });

  const [opened, { toggle, close }] = useDisclosure(false);

  if (!user?.roles.includes("ec")) return null;

  return (
    <>
      <ActionIcon variant="subtle" aria-label="Settings" ml={ml} onClick={toggle}>
        <IconPlus size={18} />
      </ActionIcon>
      <Modal opened={opened} onClose={close} title="Create slots">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            promiseWithToast(form.handleSubmit());
          }}
        >
          <Stack>
            <form.Field
              name="name"
              children={(field) => (
                <TextInput
                  label="Airspace name"
                  onChange={(e) => field.handleChange(e.target.value)}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                ></TextInput>
              )}
            />
            <Button variant="subtle" type="submit">
              Create
            </Button>
          </Stack>
        </form>
      </Modal>
    </>
  );
};
