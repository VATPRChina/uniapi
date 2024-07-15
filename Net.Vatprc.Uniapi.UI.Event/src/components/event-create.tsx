import { DateTime } from "./datetime";
import NoEventImage from "@/assets/no-event-image.svg";
import { useApi, useApiPost } from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import {
  ActionIcon,
  Button,
  Group,
  Image,
  MantineSpacing,
  Modal,
  Stack,
  StyleProp,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { useDisclosure } from "@mantine/hooks";
import { IconEdit } from "@tabler/icons-react";
import { useForm } from "@tanstack/react-form";
import { formatISO, setSeconds } from "date-fns";

const NULL_ULID = "01J2N4V2BNSP3E5Q9MBA3AE8E3";
export const CreateEvent = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId?: string }) => {
  const user = useUser();
  const { data: event, isLoading } = useApi(`/api/events/{eid}`, {
    path: { eid: eventId ?? NULL_ULID },
    enabled: !!eventId,
  });
  const { mutate: create, isPending: isCreatePending } = useApiPost("/api/events", {}, () => close());
  const { mutate: update, isPending: isUpdatePending } = useApiPost(
    "/api/events/{eid}",
    {
      path: { eid: eventId ?? NULL_ULID },
    },
    () => close(),
  );
  const form = useForm({
    defaultValues: {
      title: event?.title ?? "",
      start_at: event?.start_at ?? formatISO(setSeconds(Date.now(), 0)),
      end_at: event?.end_at ?? formatISO(setSeconds(Date.now(), 0)),
      start_booking_at: event?.start_booking_at ?? formatISO(setSeconds(Date.now(), 0)),
      end_booking_at: event?.end_booking_at ?? formatISO(setSeconds(Date.now(), 0)),
      image_url: event?.image_url ?? null,
    },
    onSubmit: ({ value }) => {
      if (eventId) {
        update(value);
      } else {
        create(value);
      }
    },
  });

  const [opened, { toggle, close }] = useDisclosure(false);

  if (!user.roles.includes("ec")) return null;

  return (
    <>
      {eventId ? (
        <ActionIcon variant="subtle" aria-label="Settings" ml={ml} onClick={toggle}>
          <IconEdit size={18} />
        </ActionIcon>
      ) : (
        <Button onClick={toggle}>Create Event</Button>
      )}
      <Modal opened={opened} onClose={close} size="xl">
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
                  disabled={isLoading}
                ></TextInput>
              )}
            />
            <Group grow>
              <form.Field
                name="start_at"
                children={(field) => (
                  <Stack gap="xs">
                    <DateTimePicker
                      label="Start at"
                      onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                      valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                      value={new Date(field.state.value)}
                      onBlur={field.handleBlur}
                      disabled={isLoading}
                    />
                    <Text size="xs">
                      <DateTime>{field.state.value}</DateTime>
                    </Text>
                  </Stack>
                )}
              />
              <form.Field
                name="end_at"
                children={(field) => (
                  <Stack gap="xs">
                    <DateTimePicker
                      label="End at"
                      onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                      valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                      value={new Date(field.state.value)}
                      onBlur={field.handleBlur}
                      disabled={isLoading}
                    />
                    <Text size="xs">
                      <DateTime>{field.state.value}</DateTime>
                    </Text>
                  </Stack>
                )}
              />
            </Group>
            <Group grow>
              <form.Field
                name="start_booking_at"
                children={(field) => (
                  <Stack gap="xs">
                    <DateTimePicker
                      label="Start booking at"
                      onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                      valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                      value={new Date(field.state.value)}
                      onBlur={field.handleBlur}
                      disabled={isLoading}
                    />
                    <Text size="xs">
                      <DateTime>{field.state.value}</DateTime>
                    </Text>
                  </Stack>
                )}
              />
              <form.Field
                name="end_booking_at"
                children={(field) => (
                  <Stack gap="xs">
                    <DateTimePicker
                      label="End booking at"
                      onChange={(e) => field.handleChange(formatISO(e ?? new Date()))}
                      valueFormat="YYYY-MM-DD HH:mm:ss ZZ"
                      value={new Date(field.state.value)}
                      onBlur={field.handleBlur}
                      disabled={isLoading}
                    />
                    <Text size="xs">
                      <DateTime>{field.state.value}</DateTime>
                    </Text>
                  </Stack>
                )}
              />
            </Group>
            <form.Field
              name="image_url"
              children={(field) => (
                <TextInput
                  label="Image URL"
                  onChange={(e) => field.handleChange(e.target.value || null)}
                  value={field.state.value ?? ""}
                  onBlur={field.handleBlur}
                  disabled={isLoading}
                />
              )}
            />
            <form.Subscribe selector={(state) => state.values.image_url}>
              {(image_url) => <Image src={image_url ?? NoEventImage} />}
            </form.Subscribe>
            <Button variant="subtle" type="submit" loading={isCreatePending || isUpdatePending}>
              {eventId ? "Save" : "Create"}
            </Button>
          </Stack>
        </form>
      </Modal>
    </>
  );
};
