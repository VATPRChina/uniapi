import { invalidatePath, useApi, useApiPost, useApiPut } from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import {
  ActionIcon,
  Button,
  MantineSpacing,
  Modal,
  Stack,
  StyleProp,
  TagsInput,
  TextInput,
  Textarea,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconEdit, IconPlus } from "@tabler/icons-react";
import { useForm } from "@tanstack/react-form";

const NULL_ULID = "01J2N4V2BNSP3E5Q9MBA3AE8E3";
export const CreateAirspace = ({
  ml,
  eventId,
  airspaceId,
}: {
  ml?: StyleProp<MantineSpacing>;
  eventId: string;
  airspaceId?: string;
}) => {
  const [opened, { toggle, close }] = useDisclosure(false);

  const user = useUser();
  const { data: airspace, isLoading } = useApi(`/api/events/{eid}/airspaces/{aid}`, {
    path: { eid: eventId, aid: airspaceId ?? NULL_ULID },
    enabled: !!airspaceId && opened,
  });
  const onComplete = () => {
    close();
    return invalidatePath(`/api/events/{eid}/airspaces`, { eid: eventId });
  };
  const { mutate: create, isPending: isCreatePending } = useApiPost(
    "/api/events/{eid}/airspaces",
    { path: { eid: eventId } },
    onComplete,
  );
  const { mutate: update, isPending: isUpdatePending } = useApiPut(
    "/api/events/{eid}/airspaces/{aid}",
    { path: { eid: eventId, aid: airspaceId ?? NULL_ULID } },
    onComplete,
  );
  const form = useForm({
    defaultValues: {
      name: airspace?.name ?? "",
      icao_codes: airspace?.icao_codes ?? ([] as string[]),
      description: airspace?.description ?? "",
    },
    onSubmit: ({ value }) => {
      if (airspaceId) return update(value);
      else return create(value);
    },
  });

  if (!user.roles.includes("ec")) return null;

  return (
    <>
      <ActionIcon variant="subtle" aria-label="Settings" ml={ml} onClick={toggle}>
        {airspaceId ? <IconEdit size={18} /> : <IconPlus size={18} />}
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
                  disabled={isLoading}
                />
              )}
            />
            <form.Field
              name="icao_codes"
              children={(field) => (
                <TagsInput
                  label="Related ICAO codes"
                  onChange={(e) => field.handleChange(e)}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  disabled={isLoading}
                />
              )}
            />
            <form.Field name="description">
              {(field) => (
                <Textarea
                  label="Description"
                  onChange={(e) => field.handleChange(e.target.value)}
                  value={field.state.value ?? ""}
                  onBlur={field.handleBlur}
                  disabled={isLoading}
                  autosize
                />
              )}
            </form.Field>
            <Button variant="subtle" type="submit" loading={isCreatePending || isUpdatePending}>
              {airspaceId ? "Save" : "Create"}
            </Button>
          </Stack>
        </form>
      </Modal>
    </>
  );
};
