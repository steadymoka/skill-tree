export const ko = {
  common: {
    appName: "Skill Tree",
    appDescription: "스킬 관리 도구",
    notInstalled: "Skill Tree가 초기화되지 않았습니다",
    notInstalledDesc: "'skill-tree init'을 실행하여 스킬 저장소를 초기화하세요.",
    empty: "데이터 없음",
  },
  skills: {
    title: "Skill Tree",
    initialize: "Skill Tree 초기화",
    initializeDesc: "기존 스킬을 복사하고 skills.yaml을 생성합니다",
    noSkills: "스킬이 없습니다",
    allTags: "전체",
    editTags: "태그 편집",
    save: "저장",
    cancel: "취소",
    link: "연결",
    unlink: "해제",
    linked: "연결됨",
    bulkLink: "태그로 일괄 연결",
    projects: "프로젝트",
    noProjects: "프로젝트가 없습니다",
    chars: "자",
    skills: "skills",
    tags: "tags",
    viewContent: "내용 보기",
    collapse: "접기",
  },
};

export type Messages = typeof ko;
