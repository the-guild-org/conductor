import { Feature } from './common/feature'
import { Highlights } from './common/highlight'
import { Ribbon } from './common/ribbon'
import {
  FiCloudLightning,
  FiUserCheck,
  FiAlertTriangle,
  FiMoreHorizontal,
  FiGlobe,
  FiCloudSnow,
  FiTarget,
  FiArrowRightCircle,
  FiLink,
  FiShuffle,
  FiList,
} from 'react-icons/fi'
import { ExamplesView } from './examples-view'
import { FlowManyChart } from './common/flow-many'

export function IndexPage() {
  return (
    <>
      <div className='w-full overflow-x-hidden'>
        <Ribbon>now on alpha</Ribbon>
        <div className='pt-40'>
          <h2 className='max-w-screen-md mx-auto font-extrabold text-5xl text-center bg-gradient-to-r from-teal-700 to-teal-900 bg-clip-text text-transparent'>
            A Fully-Featured,
            <br />
            Open-Source GraphQL Gateway for Any Project
          </h2>
        </div>
      </div>
      <div className='flex flex-col'>
        <div className='w-full py-12'>
          <div className='container box-border px-6 mx-auto flex flex-col gap-y-24'>
            <div className='flex items-center text-center flex-col'>
              <div className='px-8 text-xl'>
                <span className='font-bold'>
                  Conductor acts as a proxy between your GraphQL consumers and
                  your GraphQL server(s).
                </span>
                <br />
                <br />
                Conductor enriches your GraphQL runtime with powerful features
                such as caching,
                <br />
                rate-limiting, federated schemas and monitoring with a single
                line of configuration code.
              </div>
              <div className='mt-8'>
                <FlowManyChart />
              </div>
            </div>
            <Feature
              title='Many-to-many'
              description={
                <div>
                  Conductor can load many GraphQL source(s) and, combine,
                  transform and federate the GraphQL schemas.
                  <br />
                  You can also expose many GraphQL endpoints from a single
                  instance, with different schemas, variations and plugins.
                </div>
              }
            >
              <Highlights
                className='flex-col md:flex-col'
                items={[
                  {
                    title: 'Runs Everywhere',
                    description: (
                      <p className='text-gray-600 dark:text-gray-400'>
                        Conductor runs on your own infrastructure, or in all
                        Cloud service. You can run it as a binary or in a
                        containerized environment like Kubernetes or Docker. You
                        can also run Conductor as a CloudFlare Worker (WASM).
                      </p>
                    ),
                    icon: <FiCloudSnow size={36} />,
                  },
                  {
                    title: 'Federated Execution',
                    description:
                      'Federate, merge and transform GraphQL types from multiple sources into a unified GraphQL schema. Powered by Fusion.',
                    icon: <FiShuffle size={36} />,
                  },
                  {
                    title: 'Multiple Endpoints',
                    description:
                      'Expose the same GraphQL schema on multiple endpoints: each endpoint can have its own flow and plugins',
                    icon: <FiList size={36} />,
                  },
                  {
                    title: 'Chainable Configuration',
                    description:
                      'Every part of the execution chain is publishable: you can decide what and how to expose with every endpoint.',
                    icon: <FiLink size={36} />,
                  },
                ]}
              />
            </Feature>
            <Feature
              title='Unlimited Extensibility'
              flipped
              description={
                <>
                  <div>
                    <strong>GraphQL features with zero effort.</strong> Choose
                    from a curated list of plugins, or develop and deploy your
                    own.
                  </div>
                  <Highlights
                    className='mt-6 flex-col md:flex-col'
                    items={[
                      {
                        title: 'Response Caching',
                        description:
                          'Add caching to your GraphQL service with zero effort and no code changes.',
                        icon: <FiCloudLightning size={36} />,
                      },
                      {
                        title: 'Security',
                        description:
                          'Built-in plugins for popular authentication flows (Basic/JWT/Auth0/...). Also, hardening plugins like rate-limit are available.',
                        icon: <FiUserCheck size={36} />,
                      },
                      {
                        title: 'Monitoring',
                        description:
                          'Monitor your service with built-in support for Prometheus, StatD and OpenTelemetry.',
                        icon: <FiAlertTriangle size={36} />,
                      },
                      {
                        title: 'GraphQL to REST',
                        description: (
                          <p className='text-gray-600 dark:text-gray-400'>
                            Expose any GraphQL schemas as REST service, powered
                            by{' '}
                            <a
                              href='https://www.the-guild.dev/graphql/sofa-api'
                              target='_blank'
                            >
                              <strong>SOFA</strong>
                            </a>
                          </p>
                        ),
                        icon: <FiArrowRightCircle size={36} />,
                      },
                      {
                        title: 'and many more',
                        icon: <FiMoreHorizontal size={36} />,
                      },
                    ]}
                  />
                </>
              }
              side={<ExamplesView />}
            />
          </div>
        </div>
      </div>
    </>
  )
}
