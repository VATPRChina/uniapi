import client from "@/client";
import { promiseWithToast, useClientQuery, wrapPromiseWithToast } from "@/utils";
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
} from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@tanstack/react-form";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { addMinutes, format, formatISO, setSeconds } from "date-fns";
import { useState } from "react";
import { IoAdd } from "react-icons/io5";

export const CreateSlot = ({ ml, eventId }: { ml?: StyleProp<MantineSpacing>; eventId: string }) => {
  const queryClient = useQueryClient();
  const [rows, setRows] = useState([] as { airspace_name: string; enter_at: Date }[]);
  const { data: airspaces } = useClientQuery("/api/events/{eid}/airspaces", { params: { path: { eid: eventId } } });
  const { isPending, mutateAsync } = useMutation({
    mutationKey: ["/api/events/{eid}/slots", eventId],
    mutationFn: () =>
      Promise.all(
        rows.map((row) =>
          client.POST("/api/events/{eid}/slots", {
            params: { path: { eid: eventId } },
            body: { airspace_id: row.airspace_name, enter_at: formatISO(row.enter_at) },
          }),
        ),
      ),
    onSuccess: () => {
      close();
      return queryClient.invalidateQueries({ queryKey: ["/api/events/{eid}/slots", eventId] });
    },
  });

  const form = useForm({
    defaultValues: {
      airspaceId: "",
      start_at: formatISO(setSeconds(Date.now(), 0)),
      interval: 5,
      count: 0,
    },
    onSubmit: ({ value }) => {
      setRows(
        [...Array(value.count).keys()].map((offset) => ({
          airspace_name: value.airspaceId,
          enter_at: addMinutes(new Date(value.start_at), offset * value.interval),
        })),
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
              name="airspaceId"
              children={(field) => (
                <Select
                  label="Airspace name"
                  data={airspaces?.map((a) => ({ value: a.id, label: a.name }))}
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e ?? "")}
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
                  <Table.Td>{row.airspace_name}</Table.Td>
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
