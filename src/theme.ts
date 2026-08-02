import { createTheme, NavLink } from "@mantine/core";

import "./fonts/fonts.css";

export const theme = createTheme({
  primaryColor: "violet",
  defaultRadius: "md",
  autoContrast: true,
  respectReducedMotion: false,
  fontFamily: "Lato, sans-serif",
  headings: {
    fontFamily: "Lato, sans-serif",
    fontWeight: "700",
  },
  components: {
    // Round the hover/active background on every NavLink so it reads as a
    // pill instead of a sharp rectangle. Ties to defaultRadius above.
    NavLink: NavLink.extend({
      styles: {
        root: { borderRadius: "var(--mantine-radius-default)" },
      },
    }),
  },
});