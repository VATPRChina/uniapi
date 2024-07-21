import { TypographyStylesProvider } from "@mantine/core";
import dompurify from "dompurify";
import { parse } from "marked";

dompurify.addHook("afterSanitizeAttributes", (node) => {
  // add target=_blank to a
  if (node.nodeName.toLowerCase() === "a") {
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noopener noreferrer");
  }
});

export const Markdown = ({ children }: { children?: string }) => {
  if (!children) return null;

  const html = dompurify.sanitize(parse(children, { async: false, gfm: true, breaks: true }) as string, {});

  return (
    <TypographyStylesProvider>
      <div dangerouslySetInnerHTML={{ __html: html }} />
    </TypographyStylesProvider>
  );
};
