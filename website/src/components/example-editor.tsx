import { PropsWithChildren } from 'react';
import { useTheme } from 'next-themes';
import MonacoEditor from '@monaco-editor/react';

export const canUseDOM = typeof window !== 'undefined';

export default function ExampleEditor(
  props: PropsWithChildren<{
    lang: string;
    value: string;
    editorHeight?: string;
  }>,
) {
  const { resolvedTheme } = useTheme();

  if (!canUseDOM) {
    return null;
  }

  return (
    <MonacoEditor
      height={props.editorHeight || '40vh'}
      theme={resolvedTheme === 'dark' ? 'vs-dark' : 'vs'}
      language={props.lang}
      value={props.value}
      options={{
        readOnly: true,
        domReadOnly: true,
        selectionHighlight: false,
        lineNumbers: 'off',
        hideCursorInOverviewRuler: true,
        cursorStyle: 'line',
        contextmenu: false,
        overviewRulerBorder: false,
        overviewRulerLanes: 0,
        rulers: [],
        scrollbar: {
          vertical: 'hidden',
          horizontal: 'hidden',
          handleMouseWheel: false,
        },
        folding: false,
        minimap: {
          enabled: false,
        },
      }}
    />
  );
}
