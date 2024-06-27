import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "./ui/dialog";
import { Input } from "./ui/input";
import client from "@/client";
import { Label } from "@radix-ui/react-label";
import { useForm } from "@tanstack/react-form";

export const CreateEvent = () => {
  const form = useForm({
    defaultValues: {
      title: "",
      start_at: "",
      end_at: "",
    },
    onSubmit: async ({ value }) => {
      // Do something with form data
      // await devLogin(value.username, "foobar");
      await client.POST("/api/events", { body: value });
    },
  });

  return (
    <Dialog>
      <DialogTrigger>Create Event</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Event</DialogTitle>
        </DialogHeader>
        <form
          className="grid gap-4"
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            // eslint-disable-next-line no-console
            form.handleSubmit().catch(console.error);
          }}
        >
          <div className="grid gap-2">
            <div className="grid grid-cols-3 items-center gap-4">
              <Label htmlFor="title">Title</Label>
              <form.Field
                name="title"
                children={(field) => (
                  <Input id="title" className="col-span-2 h-8" onChange={(e) => field.handleChange(e.target.value)} />
                )}
              />
            </div>
            <div className="grid grid-cols-3 items-center gap-4">
              <Label htmlFor="start_at">Start at</Label>
              <form.Field
                name="start_at"
                children={(field) => (
                  <Input
                    id="start_at"
                    className="col-span-2 h-8"
                    onChange={(e) => field.handleChange(e.target.value)}
                  />
                )}
              />
            </div>
            <div className="grid grid-cols-3 items-center gap-4">
              <Label htmlFor="end_at">End at</Label>
              <form.Field
                name="end_at"
                children={(field) => (
                  <Input id="end_at" className="col-span-2 h-8" onChange={(e) => field.handleChange(e.target.value)} />
                )}
              />
            </div>
          </div>
          <Button variant="outline">Create</Button>
        </form>
      </DialogContent>
    </Dialog>
  );
};
