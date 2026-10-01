/** علامة الترقين: وردة ثمانية زي فواصل الأقسام في المخطوطات. */
export function Mark({ size = 20 }: { size?: number }) {
  return (
    <svg
      className="mark__rosette"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M12 1.5 15 6l5-1.2L18.8 10l4.2 2-4.2 2 1.2 5.2L15 18l-3 4.5L9 18l-5 1.2L5.2 14 1 12l4.2-2L4 4.8 9 6l3-4.5Z"
        fill="currentColor"
        opacity="0.9"
      />
      <circle cx="12" cy="12" r="3.1" fill="var(--ink-900)" />
    </svg>
  );
}
