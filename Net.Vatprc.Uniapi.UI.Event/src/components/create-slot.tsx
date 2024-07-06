import { CreateAirspace } from "./create-airspace";
import client, { formatPath, useApi } from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithToast, wrapPromiseWithToast } from "@/utils";
import {
  ActionIcon,
  Button,
  MantineSpacing,
  Modal,
  NumberInput,
  ScrollArea,
  Select,
  Stack,
  StyleProp,
  Table,
  Tooltip,
} from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@tanstack/react-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { addMinutes, format, formatISO, setSeconds } from "date-fns";
import { useState } from "react";
import { IoAdd } from "react-icons/io5";

export const CreateSlot = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId: string }) => {
  const user = useUser();
  const [rows, setRows] = useState([] as { airspace_id: string; airspace_name: string; enter_at: Date }[]);
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: eventId } });
  const { data: airspaces } = useApi("/api/events/{eid}/airspaces", { path: { eid: eventId } });
  const queryClient = useQueryClient();
  const { isPending, mutateAsync } = useMutation({
    mutationKey: ["api", "events", eventId, "slots"],
    mutationFn: () =>
      Promise.all(
        rows.map((row) =>
          client.POST("/api/events/{eid}/slots", {
            params: { path: { eid: eventId } },
            body: { airspace_id: row.airspace_id, enter_at: formatISO(row.enter_at) },
          }),
        ),
      ),
    onSuccess: () => {
      close();
      return queryClient.invalidateQueries({ queryKey: formatPath("/api/events/{eid}/slots", { eid: eventId }) });
    },
  });

  const form = useForm({
    defaultValues: {
      airspaceId: "",
      start_at: formatISO(event?.start_at ?? setSeconds(Date.now(), 0)),
      interval: 5,
      count: 0,
    },
    onSubmit: ({ value }) => {
      setRows(
        [...Array(value.count).keys()].map((offset) => ({
          airspace_id: value.airspaceId,
          airspace_name: airspaces?.find((a) => a.id === value.airspaceId)?.name ?? "",
          enter_at: addMinutes(new Date(value.start_at), offset * value.interval),
        })),
      );
    },
  });

  const [opened, { toggle, close }] = useDisclosure(false);

  if (!user?.roles.includes("ec")) return null;
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
              name="airspaceId"
              children={(field) => (
                <Select
                  label="Airspace name"
                  data={airspaces?.map((a) => ({ value: a.id, label: a.name }))}
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e ?? "")}
                  rightSection={<CreateAirspace eventId={eventId} />}
                  rightSectionPointerEvents="auto"
                />
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
              name="interval"
              children={(field) => (
                <NumberInput
                  label="Interval"
                  onChange={(e) => field.handleChange(typeof e === "number" ? e : Number.parseInt(e, 10))}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  suffix=" minutes"
                />
              )}
            />
            <form.Field
              name="count"
              children={(field) => (
                <NumberInput
                  label="Count"
                  onChange={(e) => field.handleChange(typeof e === "number" ? e : Number.parseInt(e, 10))}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  suffix=" slots"
                />
              )}
            />
            <Button variant="subtle" type="submit">
              Compute
            </Button>
          </Stack>
        </form>
        <ScrollArea h={250}>
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Airspace Name</Table.Th>
                <Table.Th>Enter at</Table.Th>
                <Table.Th></Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {rows.map((row, id) => (
                <Table.Tr key={id}>
                  <Table.Td>
                    <Tooltip label={row.airspace_id}>
                      <span>{row.airspace_name}</span>
                    </Tooltip>
                  </Table.Td>
                  <Table.Td>{format(row.enter_at, "yyyy-MM-dd HH:mm zzzz")}</Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </ScrollArea>
        <Button loading={isPending} onClick={() => wrapPromiseWithToast(mutateAsync())}>
          Create
        </Button>
      </Modal>
    </>
  );
};
