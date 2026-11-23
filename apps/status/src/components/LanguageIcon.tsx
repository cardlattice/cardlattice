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
import Icon from "./Icon.tsx";
const flags: Record<string, string> = {
  en: "us",
  ja: "jp",
  ko: "kr",
  nl: "nl",
  "zh-Hans": "cn",
  "zh-Hant": "tw",
  fr: "fr",
  de: "de",
  es: "es",
  it: "it",
  pt: "br",
};
export default function LanguageIcon({ language }: { language: string }) {
  const flag = flags[language];
  const sources: Record<string, string> = { us, jp, kr, nl, cn, tw, fr, de, es, it, br };
  return flag ? (
    <span
      className={`fi fi-${flag} flag`}
      style={{ backgroundImage: `url(${sources[flag]})` }}
      aria-hidden="true"
    />
  ) : (
    <Icon name="globe" />
  );
}
