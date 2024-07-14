import { useApiPost } from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import { ActionIcon, Button, MantineSpacing, Modal, Stack, StyleProp, TagsInput, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconPlus } from "@tabler/icons-react";
import { useForm } from "@tanstack/react-form";

export const CreateAirspace = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId: string }) => {
  const user = useUser();
  const { mutate } = useApiPost("/api/events/{eid}/airspaces", { path: { eid: eventId } }, () => close());
  const form = useForm({
    defaultValues: {
      name: "",
      icaoCodes: [] as string[],
    },
    onSubmit: ({ value }) => {
      mutate({ name: value.name, icao_codes: value.icaoCodes });
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
                  label="Area name"
                  onChange={(e) => field.handleChange(e.target.value)}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                ></TextInput>
              )}
            />
            <form.Field
              name="icaoCodes"
              children={(field) => (
                <TagsInput
                  label="Related ICAO codes"
                  onChange={(e) => field.handleChange(e)}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                ></TagsInput>
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
