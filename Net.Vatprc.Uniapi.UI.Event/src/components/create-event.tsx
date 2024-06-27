import client from "@/client";
import { promiseWithToast } from "@/utils";
import { Button, Modal, Stack, TextInput, Title } from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@tanstack/react-form";
import { formatISO, setSeconds } from "date-fns";

export const CreateEvent = () => {
  const form = useForm({
    defaultValues: {
      title: "",
      start_at: formatISO(setSeconds(Date.now(), 0)),
      end_at: formatISO(setSeconds(Date.now(), 0)),
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
                <TextInput
                  label="Title"
                  onChange={(e) => field.handleChange(e.target.value)}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                ></TextInput>
              )}
            />
            <form.Field
              name="start_at"
              children={(field) => (
                <DateTimePicker
                  label="Start at"
                  onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                  valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                  value={new Date(field.state.value)}
                  onBlur={field.handleBlur}
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
                  value={new Date(field.state.value)}
                  onBlur={field.handleBlur}
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
