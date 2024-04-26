import cx from "classnames";
import "./Tab.css";
import { useCallback } from "react";

interface Props<T> {
  options: T[],
  option: T,
  setOption?: (option: T) => void,
  onChange?: (option: T) => void,
  type?: "block" | "inline",
  className: string,
  optionLabels: {
    [opt: string]: string,
  },
  icons?: {
    [opt: string]: string,
  }
}

export default function Tab<T extends string>({
  options,
  option,
  setOption,
  onChange,
  type = "block",
  className,
  optionLabels,
  icons,
}: Props<T>) {
  const onClick = useCallback((opt: T) => {
    if (setOption) {
      setOption(opt);
    }
    if (onChange) {
      onChange(opt);
    }
  }, [onChange, setOption]);

  return (
    <div className={cx("Tab", type, className)}>
      {options.map((opt) => {
        const label = optionLabels && optionLabels[opt] ? optionLabels[opt] : opt;
        return (
          <div className={cx("Tab-option", "muted", { active: opt === option })} onClick={() => onClick(opt)} key={opt}>
            {icons && icons[opt] && <img className="Tab-option-icon" src={icons[opt]} alt={option} />}
            {label}
          </div>
        );
      })}
    </div>
  );
}
