import client from "@/client";
import { promiseWithToast } from "@/utils";
import { ActionIcon, Button, MantineSpacing, Modal, Stack, StyleProp, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@tanstack/react-form";
import { useQueryClient } from "@tanstack/react-query";
import { IoAdd } from "react-icons/io5";

export const CreateAirspace = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId: string }) => {
  const queryClient = useQueryClient();
  const form = useForm({
    defaultValues: {
      name: "",
    },
    onSubmit: ({ value }) => {
      promiseWithToast(
        client
          .POST("/api/events/{eid}/airspaces", { params: { path: { eid: eventId } }, body: { name: value.name } })
          .then(() => {
            close();
            return queryClient.invalidateQueries({ queryKey: ["/api/events/{eid}/airspaces", eventId] });
          }),
      );
    },
  });

  const [opened, { toggle, close }] = useDisclosure(false);

  return (
    <>
      <ActionIcon variant="subtle" aria-label="Settings" ml={ml} onClick={toggle}>
        <IoAdd />
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
