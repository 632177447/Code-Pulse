export interface SettingItem {
  id: string;
  type: 'slider' | 'input' | 'textarea' | 'switch' | 'radio' | 'checkbox';
  inputType?: 'text' | 'number' | 'password';
  label?: string;
  description?: string;
  placeholder?: string;
  min?: number;
  max?: number;
  step?: number;
  rows?: number;
  layout?: 'grid' | 'flex';
  columns?: number;
  options?: Array<{ label: string; value: any }>;
  visible?: (settings: any) => boolean;
}

export interface SettingGroup {
  id: string;
  title: string;
  colorClass?: string;
  color?: string;
  items: SettingItem[];
}
