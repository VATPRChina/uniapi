import { TypographyStylesProvider } from "@mantine/core";
import dompurify from "dompurify";
import { Marked } from "marked";
import { DirectiveConfig, createDirectives } from "marked-directive";
import { first, last } from "radash";

const fplDirective: DirectiveConfig = {
  level: "block",
  marker: "::",
  renderer(token) {
    if (token.meta.name === "fpl") {
      const routeSeg = token.text.trim().split(" ");
      const dep = first(routeSeg);
      const arr = last(routeSeg);
      const route = routeSeg.slice(1, -1).join(" ");

      const fullRouteEnc = encodeURIComponent(`${dep} ${route} ${arr}`);
      const routeEnc = encodeURIComponent(route);
      return `<p><strong>${dep}</strong> ${route} <strong>${arr}</strong>
        <br>
        <a rel="noopener noreferrer nofollow" href="https://my.vatsim.net/pilots/flightplan?route=${routeEnc}&departure=${dep}&arrival=${arr}" target="_blank">VATSIM Prefile</a>
        <span> | </span>
        <a rel="noopener noreferrer nofollow" href="https://skyvector.com/?fpl=${fullRouteEnc}" target="_blank">SkyVector</a>
        <span> | </span>
        <a rel="noopener noreferrer nofollow" href="https://www.simbrief.com/system/dispatch.php?orig=${dep}&dest=${arr}&route=${routeEnc}" target="_blank">SimBrief</a>
      </p>`;
    }

    return false;
  },
};

const parser = new Marked().use(createDirectives([fplDirective]));

dompurify.addHook("afterSanitizeAttributes", (node) => {
  // add target=_blank to a
  if (node.nodeName.toLowerCase() === "a") {
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noopener noreferrer");
  }
});

export const Markdown = ({ children }: { children?: string }) => {
  if (!children) return null;

  const html = dompurify.sanitize(parser.parse(children, { async: false, gfm: true, breaks: true }) as string, {});

  return (
    <TypographyStylesProvider>
      <div dangerouslySetInnerHTML={{ __html: html }} />
    </TypographyStylesProvider>
  );
};
