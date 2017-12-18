package problem

//Problem represents the interface of problem
type Problem interface {
	GetContestSiteName() string
	GetContestName() (string, error)
	GetProblemName() (string, error)
	GetTimeLimit() (string, error)
	GetMemoryLimit() (string, error)
	GetSampleInputs() ([]string, error)
	GetSampleOutpus() ([]string, error)
}
