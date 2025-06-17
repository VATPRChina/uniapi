import { DateTime } from "./datetime";
import client, { formatPath, queryClient } from "@/client";
import { useUser } from "@/services/auth";
import { wrapPromiseWithToast } from "@/utils";
import { ActionIcon, Button, Group, Modal, Pill, Stack, Table, Text, useMantineTheme } from "@mantine/core";
import { Dropzone, FileWithPath } from "@mantine/dropzone";
import { useDisclosure } from "@mantine/hooks";
import { IconFileImport, IconFileTypeCsv, IconUpload, IconX } from "@tabler/icons-react";
import { useMutation } from "@tanstack/react-query";
import { parse } from "date-fns";
import { fromZonedTime } from "date-fns-tz";
import { unique } from "radash";
import { useState } from "react";

interface Slot {
  airspace: string;
  icao_codes: string[];
  enter_at: Date;
  leave_at?: Date;
  callsign?: string;
  aircraft_type_icao?: string;
}

export const ImportSlot = ({ eventId }: { eventId: string }) => {
  const [opened, { toggle, close }] = useDisclosure(false);
  const theme = useMantineTheme();
  const user = useUser();

  const [slots, setSlots] = useState<Slot[]>([]);

  const onDrop = async (files: FileWithPath[]) => {
    const file = files[0];
    const data = (await file?.text()) ?? "";
    setSlots(
      data
        .split("\n")
        .map((line) => {
          const [dep, dep_time, arr, arr_time, callsign, aircraft_type_icao] = line.split(",");
          if (!dep || !dep_time || !arr) return;
          return {
            airspace: `${dep} - ${arr}`,
            icao_codes: [dep, arr],
            enter_at: fromZonedTime(parse(dep_time ?? "", "yyyy-MM-dd HH:mm", Date.now()), "UTC"),
            leave_at: arr_time ? fromZonedTime(parse(arr_time, "yyyy-MM-dd HH:mm", Date.now()), "UTC") : undefined,
            callsign: callsign,
            aircraft_type_icao: aircraft_type_icao,
          };
        })
        .filter((x) => !!x),
    );
  };

  const { isPending, mutate } = useMutation({
    mutationKey: formatPath("/api/events/{eid}/slots", { eid: eventId }),
    mutationFn: async () => {
      const airspaces = await Promise.all(
        unique(slots, (s) => s.airspace).map((slot) =>
          client.POST("/api/events/{eid}/airspaces", {
            params: { path: { eid: eventId } },
            body: { name: slot.airspace, icao_codes: slot.icao_codes, description: "" },
          }),
        ),
      );
      await Promise.all(
        slots.map((slot) =>
          client.POST("/api/events/{eid}/slots", {
            params: { path: { eid: eventId } },
            body: {
              airspace_id: airspaces.find((a) => a.data?.name === slot.airspace)?.data?.id ?? "",
              enter_at: slot.enter_at.toISOString(),
              leave_at: slot.leave_at?.toISOString(),
              callsign: slot.callsign,
              aircraft_type_icao: slot.aircraft_type_icao,
            },
          }),
        ),
      );
    },
    onSuccess: () => {
      close();
      return queryClient.invalidateQueries({ queryKey: formatPath("/api/events/{eid}", { eid: eventId }) });
    },
  });
  const onSubmit = () => {
    mutate();
  };

  if (!user.roles.includes("event_coordinator")) return null;
  return (
    <>
      <ActionIcon variant="subtle" aria-label="Settings" onClick={toggle}>
        <IconFileImport size={18} />
      </ActionIcon>
      <Modal opened={opened} onClose={close} title="Import slots" size="xl">
        <Stack>
          <Text>
            File format: CSV with dep, dep_time (yyyy-MM-dd HH:mm), arr, arr_time (yyyy-MM-dd HH:mm), callsign,
            aircraft_type_icao
          </Text>
          <Dropzone onDrop={(f) => wrapPromiseWithToast(onDrop(f))}>
            <Group justify="center" gap="xl" style={{ pointerEvents: "none" }}>
              <Dropzone.Accept>
                <IconUpload color={theme.colors.green[7]} size={52} stroke={1.5} />
              </Dropzone.Accept>
              <Dropzone.Reject>
                <IconX color={theme.colors.red[7]} size={52} stroke={1.5} />
              </Dropzone.Reject>
              <Dropzone.Idle>
                <IconFileTypeCsv color={theme.colors.gray[7]} size={52} stroke={1.5} />
              </Dropzone.Idle>

              <Text size="xl" inline>
                Drag CSV here or click to select files
              </Text>
            </Group>
          </Dropzone>
          <Button onClick={onSubmit} disabled={slots.length === 0} loading={isPending}>
            Create Slots
          </Button>
          <Table highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Area</Table.Th>
                <Table.Th>Enter at</Table.Th>
                <Table.Th>Callsign</Table.Th>
                <Table.Th>Aircraft Type</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {slots?.map((slot, i) => (
                <Table.Tr key={i}>
                  <Table.Td>{slot.airspace}</Table.Td>
                  <Table.Td>
                    <Stack>
                      <Text>
                        <Pill mr="xs">CTOT</Pill>
                        <DateTime noDate>{slot.enter_at}</DateTime>
                      </Text>
                      <Text>
                        <Pill mr="xs">ELDT</Pill>
                        <DateTime noDate>{slot.leave_at}</DateTime>
                      </Text>
                    </Stack>
                  </Table.Td>
                  <Table.Td>{slot.callsign}</Table.Td>
                  <Table.Td>{slot.aircraft_type_icao}</Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Stack>
      </Modal>
    </>
  );
};
