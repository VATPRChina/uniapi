import client from "@/client";
import { promiseWithToast } from "@/utils";
import { Button, Modal, Stack, TextInput, Title } from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@tanstack/react-form";
import { formatISO } from "date-fns";

export const CreateEvent = () => {
  const form = useForm({
    defaultValues: {
      title: "",
      start_at: "",
      end_at: "",
    },
    onSubmit: async ({ value }) => {
      await client.POST("/api/events", { body: value });
    },
  });

  const [opened, { toggle, close }] = useDisclosure(false);

  return (
    <>
      <Button onClick={toggle}>Create Event</Button>
      <Modal opened={opened} onClose={close}>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            promiseWithToast(form.handleSubmit());
          }}
        >
          <Stack>
            <Title order={4}>Create Event</Title>
            <form.Field
              name="title"
              children={(field) => (
                <TextInput label="Title" onChange={(e) => field.handleChange(e.target.value)}></TextInput>
              )}
            />
            <form.Field
              name="start_at"
              children={(field) => (
                <DateTimePicker
                  label="Start at"
                  onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                  valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                />
              )}
            />
            <form.Field
              name="end_at"
              children={(field) => (
                <DateTimePicker
                  label="End at"
                  onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                  valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                />
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
