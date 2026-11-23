import { languageName } from "../lib/model.ts";
import LanguageIcon from "./LanguageIcon.tsx";
import SelectPicker from "./SelectPicker.tsx";

export default function LanguagePicker({
  languages,
  value,
  onChange,
}: {
  languages: string[];
  value: string;
  onChange: (value: string) => void;
}) {
  const options = ["", ...languages].map((language) => ({
    value: language,
    label: language ? languageName(language) : "All languages",
    icon: <LanguageIcon language={language} />,
  }));
  return (
    <SelectPicker
      label="Language"
      value={value}
      onChange={onChange}
      options={options}
      triggerLabel={value ? languageName(value) : "Language"}
    />
  );
}
