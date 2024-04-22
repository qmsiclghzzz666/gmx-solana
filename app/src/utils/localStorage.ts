import { useLocalStorage } from "react-use";

export type LocalStorageKey = string | number | boolean | null | undefined;

export function useLocalStorageSerializeKey<T>(
  key: LocalStorageKey | LocalStorageKey[],
  initialValue: T,
  opts?: {
    raw: boolean;
    serializer: (val: T) => string;
    deserializer: (value: string) => T;
  }
) {
  key = JSON.stringify(key);

  return useLocalStorage<T>(key, initialValue, opts);
}
