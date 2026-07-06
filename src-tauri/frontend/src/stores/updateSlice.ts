export interface UpdateSlice {
  updateAvailable: boolean;
  updateStatus: string;
  updateProgress: number;

  setUpdateStatus: (available: boolean, status?: string) => void;
  setUpdateProgress: (progress: number) => void;
}

export const createUpdateSlice = (set: any) => ({
  updateAvailable: false,
  updateStatus: "",
  updateProgress: 0,

  setUpdateStatus: (available: boolean, status?: string) => set({ updateAvailable: available, updateStatus: status || "" }),
  setUpdateProgress: (progress: number) => set({ updateProgress: progress }),
});
