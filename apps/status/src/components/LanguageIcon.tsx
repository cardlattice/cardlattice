import * as stylex from "@stylexjs/stylex";
import { styles } from "../styles.stylex.ts";
import GlobeIcon from "./icons/GlobeIcon.tsx";
import us from "flag-icons/flags/4x3/us.svg";
import jp from "flag-icons/flags/4x3/jp.svg";
import kr from "flag-icons/flags/4x3/kr.svg";
import nl from "flag-icons/flags/4x3/nl.svg";
import cn from "flag-icons/flags/4x3/cn.svg";
import tw from "flag-icons/flags/4x3/tw.svg";
import fr from "flag-icons/flags/4x3/fr.svg";
import de from "flag-icons/flags/4x3/de.svg";
import es from "flag-icons/flags/4x3/es.svg";
import it from "flag-icons/flags/4x3/it.svg";
import br from "flag-icons/flags/4x3/br.svg";

const flags = new Map([
  ["en", { country: "us", source: us }],
  ["ja", { country: "jp", source: jp }],
  ["ko", { country: "kr", source: kr }],
  ["nl", { country: "nl", source: nl }],
  ["zh-Hans", { country: "cn", source: cn }],
  ["zh-Hant", { country: "tw", source: tw }],
  ["fr", { country: "fr", source: fr }],
  ["de", { country: "de", source: de }],
  ["es", { country: "es", source: es }],
  ["it", { country: "it", source: it }],
  ["pt", { country: "br", source: br }],
]);

export default function LanguageIcon({ language }: { language: string }) {
  const flag = flags.get(language);
  return flag ? (
    <img
      alt=""
      src={flag.source}
      data-country={flag.country}
      {...stylex.props(styles.flag)}
      aria-hidden="true"
    />
  ) : (
    <GlobeIcon />
  );
}
