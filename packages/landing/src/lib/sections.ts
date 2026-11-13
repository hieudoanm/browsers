export const DOC_SECTIONS: { id: string; label: string }[] = [
  { id: 'guide', label: 'Guide' },
  { id: 'api', label: 'API' },
  { id: 'deployment', label: 'Deployment' },
  { id: 'project', label: 'Project' },
];

export const sectionLabel = (id: string): string => {
  return DOC_SECTIONS.find((s) => s.id === id)?.label ?? id;
};
